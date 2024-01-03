import os
import shutil

def main():
    # create folder "out" if it doesn't exist and ensure that its empty
    if os.path.exists("out"):
       if os.path.exists("out"):
        shutil.rmtree("out")
    os.makedirs("out")

    # Run compile.py in blog and wait for it to finish, get a return code
    result = os.system("python blog/compile.py")
    if result != 0:
        print('Aborting after compilation failed of blog')
        return result

    shutil.copytree("blog/out", os.path.join("out", "blog"))

    # copy file to output
    for folder_name in ["about", "faq", "material", "projects"]:
        shutil.copytree(folder_name, os.path.join("out", folder_name))
    for file_name in ["index.html", "style.css", "document-styles.css", "base-page-style.css"]:
        shutil.copy(file_name, os.path.join("out", file_name))

    # start http server in folder out and open in browser
    os.system("cd out && python -m http.server 8000")


if __name__ == "__main__":
    main()