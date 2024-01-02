import os
import shutil
from datetime import date


def main():
    os.chdir('/'.join(__file__.split("/")[:-1]))

    print("reading template...")
    with open("./index.html", "r") as f:
        template_index = f.read()
    with open("./post_template.html", "r") as f:
        template_post = f.read()

    print("setting up output directory...")
    try:
        shutil.rmtree("./out")
    except FileNotFoundError:
        pass
    os.makedirs("./out", exist_ok=True)

    print("creating blog posts...")
    post_list = []  # [(time, html)]
    for file in os.scandir("./posts"):
        name = file.name.split('.')
        if not (file.is_file() and len(name) == 2 and name[1] == 'blog'):
            print(f"WARNING: Ignoring unexpected file in posts folder: {file.name}")
            continue

        with open(file, "r") as f:
            f_content = f.readlines()

        if len(f_content) < 4:
            print(f"WARNING: Ignoring blog post {file.name} as it's too short")
            continue

        post_url_name = name[0]

        post_title = f_content[0]
        post_desc = f_content[1]
        post_time = date.fromtimestamp(int(f_content[2]))

        post_content = ''.join(f_content[3:]).replace("\n\n", "</p><p>")

        os.mkdir(f"./out/{post_url_name}")
        content = template_post.replace("{{content}}", f"<h1>{post_title}</h1><p>{post_content}</p>")
        with open(f"./out/{post_url_name}/index.html", "w") as f:
            f.write(content)

        post_list.append((int(f_content[2]), f'<a href="./{post_url_name}"><div class="post"><h1>{post_title}</h1><p>{post_desc}</p><i>{post_time.strftime("B d, Y")}</i></div></a>'))

    print("creating index...")
    for post in sorted(post_list, key=lambda p: p[0]):
        template_index = template_index.replace("{{posts}}", "{{posts}}" + post[1])
    template_index = template_index.replace("{{posts}}", "")
    with open("./out/index.html", "w") as f:
        f.write(template_index)

    print("finished!")


if __name__ == "__main__":
    main()