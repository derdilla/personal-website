import os
import sys
import shutil


# Run build.py and minify the results to the folder "out/compiled"
def main():
    print("\033[92mBuilding page:\033[0m")
    result = os.system("python build.py")
    if result != 0:
        print("\033[91mBuild failed!\033[0m")
        return result

    if '--no-merge' in sys.argv:
        print("\033[92mSkipping merge.\033[0m")
    else:
        print("\033[92mMerging results:\033[0m")
        result = os.system("python merge.py")
        if result != 0:
            print("\033[91mMerging failed!\033[0m")
            return result

    if '--no-minify' in sys.argv:
        print("\033[92mSkipping minification.\033[0m")
    elif '--no-merge' in sys.argv:
        print("\033[92mSkipping minification (no merged files).\033[0m")
    else:
        print("\033[92mMinifying results:\033[0m")
        result = os.system("python minify.py")
        if result != 0:
            print("\033[91mMinifying failed!\033[0m")
            return result

    result = os.system("python sitemapper.py")
    if result != 0:
        print("\033[91mSitemapping failed!\033[0m")
        return result

    if '--open-webserver' in sys.argv:
        print("\033[92mOpening web server.\033[0m")
        if '--no-merge' in sys.argv:
            os.system("cd out/build && python -m http.server 8000")
        elif '--no-minify' in sys.argv:
            os.system("cd out/merged && python -m http.server 8000")
        else:
            os.system("cd out/minified && python -m http.server 8000")




if __name__ == '__main__':
    main()