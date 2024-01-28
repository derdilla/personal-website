import os
import re
import shutil

def main():
    if os.path.exists("out/merged"):
        shutil.rmtree("out/merged")
    os.makedirs("out/merged")

    for root, _, files in os.walk('out/build'):
        for file in files:
            if file.endswith('.html'):
                file_path = os.path.join(root, file)
                save_path = file_path.replace('out/build', 'out/merged')
                process_html_file(file_path, save_path)

    shutil.copytree('media', 'out/merged/media')


def process_html_file(file_path, save_path):
    with open(file_path, 'r', encoding='utf-8') as file:
        content = file.read()

    # Find all CSS links and insert their content into <style> tags
    css_links = re.findall(r'<link[^>]*rel=["\']stylesheet["\'][^>]*href=["\']([^"\']+)', content)
    for css_link in css_links:
        if css_link.startswith('http'):
            continue
        if css_link.startswith('/'):
            css_content = get_css_content(os.path.join('out/build', css_link[1:]))
        else:
            css_content = get_css_content(os.path.join(os.path.dirname(file_path), css_link))
        if css_content:
            content = content.replace(f'<link rel="stylesheet" href="{css_link}">', f'<style>{css_content}</style>', 1)

    # Save the modified HTML content
    os.makedirs(os.path.dirname(save_path), exist_ok=True)
    print(f' > {save_path}')
    with open(save_path, 'w', encoding='utf-8') as file:
        file.write(content)


def get_css_content(css_path):
    if os.path.exists(css_path):
        with open(css_path, 'r', encoding='utf-8') as css_file:
            css_content = css_file.read()

        # Find and include nested CSS files
        nested_css_links = re.findall(r'@import\s+url\(([^)]+)\);', css_content)
        for nested_css_link in nested_css_links:
            nested_css_content = get_css_content(os.path.join(os.path.dirname(css_path), nested_css_link))
            css_content = re.sub(
                fr'@import\s+url\({nested_css_link}\);',
                nested_css_content,
                css_content
            )

        return css_content
    print(f"Warn: Can't find css file '{css_path}'")
    return ''

if __name__ == '__main__':
    main()