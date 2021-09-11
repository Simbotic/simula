#version 450

layout(location = 1) in vec4 v_Color;

layout(location = 0) out vec4 o_Target;

mat4 threshold = mat4(
	1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0,
	13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0,
	4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0,
	16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0
);

void main() {
    o_Target = v_Color;
    int x = int(mod(gl_FragCoord.x, 4));
	int y = int(mod(gl_FragCoord.y, 4));
	float alpha = o_Target.a - threshold[x][y];
	if (alpha < 0.0) {
		discard;
	}
}
