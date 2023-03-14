import sys
import subprocess
import os
import platform
import importlib
import bpy

def isWindows():
    return os.name == 'nt'

def isMacOS():
    return os.name == 'posix' and platform.system() == "Darwin"

def isLinux():
    return os.name == 'posix' and platform.system() == "Linux"

def findPythonBin():
    if isWindows():
        return os.path.join(sys.prefix, 'bin', 'python.exe')
    elif isMacOS():
        path = sys.executable
        sys.path.append(os.path.join(os.environ["HOME"], ".local/lib/python3.10/site-packages"))
        return os.path.abspath(path)
    elif isLinux():
        path = sys.executable
        return os.path.abspath(path)
    else:
        print("sorry, still not implemented for ", os.name, " - ", platform.system)
        
def require(packageName):
    pythonBin = findPythonBin()
    print("Python EXE:", pythonBin)
    try:
        module = importlib.import_module(packageName)
        print("Installed:", packageName)
    except:
        subprocess.call([pythonBin, "-m", "ensurepip", "--default-pip"])
        subprocess.call([pythonBin, "-m", "pip", "install", packageName])
        
def uninstall(packageName):
    pythonBin = findPythonBin()
    print("Python EXE:", pythonBin)
    subprocess.call([pythonBin, "-m", "pip", "uninstall", packageName])
