import os
import re
import shutil


# Run build.py and minify the results to the folder "out/compiled"
def main():
    print("\033[92mBuilding page:\033[0m")
    result = os.system("python build.py")
    if result != 0:
        print("\033[91mBuild failed!\033[0m")
        return result

    print("\033[92mMerging results:\033[0m")
    result = os.system("python merge.py")
    if result != 0:
        print("\033[91mMerging failed!\033[0m")
        return result

    print("\033[92mMinifying results:\033[0m")
    result = os.system("python minify.py")
    if result != 0:
        print("\033[91mMinifying failed!\033[0m")
        return result


if __name__ == '__main__':
    main()