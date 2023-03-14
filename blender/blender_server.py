import uuid
import os
import bpy

module = bpy.data.texts["module.py"].as_module()
module.require("aiohttp")
module.require("aiohttp_cors")

import asyncio
import aiohttp_cors
from aiohttp import web

async def handle(request):
    name = request.match_info.get('name', "Anonymous")
    response_json = {"message": "Hello, " + name}
    return web.json_response(response_json)

async def version(request):
    version = bpy.app.version_string
    return web.Response(text=version)

async def objects(request):
    obj_list = [{"name": obj.name, "id": obj.as_pointer(), "type": obj.type} for obj in bpy.data.objects]
    return web.json_response({"objects": obj_list})

async def object(request):
    object_name = request.match_info.get('object_name')
    obj = bpy.data.objects.get(object_name)
    if obj:
        response_json = {"object": {"name": obj.name, "id": obj.as_pointer(), "type": obj.type}}
        if obj.material_slots:
            materials = [{"name": slot.material.name, "id": slot.material.as_pointer()} for slot in obj.material_slots]
            response_json["materials"] = materials
        if obj.type == 'MESH':
            mesh = obj.data
            response_json["mesh"] = {"name": mesh.name, "id": mesh.as_pointer()}
        return web.json_response(response_json)
    else:
        return web.json_response({"error": f"Object '{object_name}' not found"}, status=404)

async def collections(request):
    collections = [{"name": col.name, "id": col.as_pointer()} for col in bpy.data.collections]
    return web.json_response({"collections": collections})

async def collection_objects(request):
    collection_name = request.match_info.get('collection_name')
    collection = bpy.data.collections.get(collection_name)
    if collection:
        obj_list = [{"name": obj.name, "id": obj.as_pointer(), "type": obj.type} for obj in collection.objects]
        return web.json_response({"objects": obj_list})
    else:
        return web.json_response({"error": f"Collection '{collection_name}' not found"}, status=404)
    
async def materials(request):
    material_list = [{"name": mat.name, "id": mat.as_pointer()} for mat in bpy.data.materials]
    return web.json_response({"materials": material_list})

async def material(request):
    material_name = request.match_info.get('material_name')
    mat = bpy.data.materials.get(material_name)
    textures = [{"name": node.label, "type": node.type} for node in mat.node_tree.nodes if node.type == 'TEX_IMAGE']
    if mat:
        response_json = {
            "material": {
                "name": mat.name, 
                "textures": textures,
                "id": mat.as_pointer(), 
                "diffuse_color": mat.diffuse_color[:],
#                "baseColorTexture": {"index": -1} if not mat.use_textures[0] else {"index": mat.active_texture_index},
                "metallic": mat.metallic,
                "roughness": mat.roughness,
#                "metallicRoughnessTexture": {"index": -1} if not mat.use_textures[2] else {"index": mat.active_texture_index},
#                "normalTexture": {"index": -1} if not mat.use_textures[1] else {"index": mat.active_texture_index},
#                "occlusionTexture": {"index": -1},
                # "emissiveFactor": mat.emit,
#                "emissiveTexture": {"index": -1} if not mat.use_textures[3] else {"index": mat.active_texture_index}
            }
        }
        return web.json_response(response_json)
    else:
        return web.json_response({"error": f"Material '{material_name}' not found"}, status=404)

async def texts(request):
    text_list = [{"name": text.name, "id": text.as_pointer()} for text in bpy.data.texts]
    return web.json_response({"texts": text_list})

async def run_script(request):
    script_name = request.match_info.get('script_name')
    script = bpy.data.texts.get(script_name)
    if script:
        override = bpy.context.copy()
        override["edit_text"] = script
        try:
            with bpy.context.temp_override(**override):
                bpy.ops.text.run_script()
        except Exception as e:
            print(e)
            return web.json_response({"error": f"Error while running Text '{script_name}'. Check the system console for error output"}, status=500)
        response_json = {
            "script": script_name,
            "status": "Success"
        }
        return web.json_response(response_json)
    else:
        return web.json_response({"error": f"Text '{script_name}' not found"}, status=404)
    
async def export_scene(request):
    scene = bpy.context.scene
    bpy.context.view_layer.objects.active = scene.objects[0]
    bpy.ops.object.select_all(action='SELECT')

    blend_file_path = bpy.data.filepath
    folder_path = os.path.dirname(blend_file_path)

    export_folder_path = os.path.join(folder_path, "models")
    if not os.path.exists(export_folder_path):
        os.makedirs(export_folder_path)

    file_name = "model-" + str(uuid.uuid4())

    export_path = os.path.join(export_folder_path, file_name + ".gltf")

    bpy.ops.export_scene.gltf(filepath=export_path,
                                check_existing=False,
                                export_format='GLTF_EMBEDDED',
                                export_tangents=True,
                                export_colors=True,
                                export_materials='EXPORT',
                                use_selection=True,
                                export_apply=True,
                                export_cameras=False,
                                export_frame_step=1,
                                export_animations=False,
                                export_lights=False)

    bpy.ops.object.select_all(action='DESELECT')
    
    return web.json_response({"file_name": file_name})

async def init_app():
    app = web.Application()
    cors = aiohttp_cors.setup(app, defaults={
        "*": aiohttp_cors.ResourceOptions(
            allow_credentials=True,
            expose_headers="*",
            allow_headers="*"
        )
    })
    app.add_routes([
        web.get('/version', version),
        web.get('/objects', objects),
        web.get('/object/{object_name}', object),
        web.get('/collections', collections),
        web.get('/collection/{collection_name}', collection_objects),
        web.get('/materials', materials),
        web.get('/material/{material_name}', material),
        web.get('/texts', texts),
        web.post('/run_script/{script_name}', run_script),
        web.post('/export_scene', export_scene),
        web.get('/{name}', handle),
    ])
    
    for route in list(app.router.routes()):
        cors.add(route)
    return app

def tick_server():
    loop.stop()
    loop.run_forever()
    return 0.1

def server_start():
    global runner, site, loop
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    app = loop.run_until_complete(init_app())
    runner = web.AppRunner(app)
    loop.run_until_complete(runner.setup())
    site = web.TCPSite(runner, 'localhost', 8080)
    loop.run_until_complete(site.start())
    bpy.app.timers.register(tick_server)
    
async def stop_server():
    await runner.cleanup()
    bpy.app.timers.unregister(tick_server)

class ServerStopButton(bpy.types.Operator):
    """Server Stop Button"""
    bl_idname = "text.server_stop_button"
    bl_label = "Stop Server"

    def execute(self, context):
        unregister()
        loop.run_until_complete(stop_server())
        loop.close()
        return {'FINISHED'}

def add_button(self, context):
    self.layout.operator(ServerStopButton.bl_idname, text="Stop Server")

def register():
    bpy.utils.register_class(ServerStopButton)
    bpy.types.TEXT_HT_header.append(add_button)

def unregister():
    bpy.types.TEXT_HT_header.remove(add_button)
    bpy.utils.unregister_class(ServerStopButton)

if __name__ == "__main__":
    server_start()
    register()
