import os
import shutil
import sys


# Runs all build scripts in subdirectories and copies results to.
# Pass argument --dev to start a http server.
def main():
    # create folder "out/build" if it doesn't exist and ensure that its empty
    if os.path.exists("out"):
        if os.path.exists("out/build"):
            shutil.rmtree("out/build")
    os.makedirs("out/build")

    # Run build.py in blog and wait for it to finish, get a return code
    result = os.system("python blog/build.py")
    if result != 0:
        print('Aborting after blog build failed')
        return result

    shutil.copytree("blog/out", os.path.join("out/build", "blog"))

    # copy file to output
    for folder_name in ["about", "faq", "material", "projects"]:
        shutil.copytree(folder_name, os.path.join("out/build", folder_name))
    for file_name in ["index.html", "style.css", "document-styles.css", "base-page-style.css"]:
        shutil.copy(file_name, os.path.join("out/build", file_name))

    # start http server in folder out/build and open in browser if the "-dev" argument is passed
    if "--dev" in sys.argv:
        os.system("cd out/build && python -m http.server 8000")


if __name__ == "__main__":
    main()
