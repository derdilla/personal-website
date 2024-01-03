import re
import os
import shutil


def main():
    if os.path.exists("out/minified"):
        shutil.rmtree("out/minified")
    for root, _, files in os.walk('out/merged'):
        for file in files:
            file_path = os.path.join(root, file)
            save_path = file_path.replace('out/merged', 'out/minified')
            minify_html_file(file_path, save_path)

def minify_html_file(file_path, save_path):
    with open(file_path, 'r', encoding='utf-8') as file:
        html_string = file.read()

    # Remove unused CSS class definitions from the HTML string
    css_classes = set(re.findall(r'\.([\w-]+)\s*{', html_string))
    used_css_classes = (e.split(' ') for e in re.findall(r'\sclass\s*=\s*[\'"]([^\'"]+)[\'"]', html_string))
    used_css_classes = set(element for inner_list in used_css_classes for element in inner_list)
    unused_css_classes = css_classes - used_css_classes
    for unused_css_class in unused_css_classes:
        html_string = re.sub(r'\.' + unused_css_class + r'\s*{[^}]+}', '', html_string)

    # Remove unused CSS variable definitions from the HTML string
    var_definitions = re.findall(r'--[\w-]+\s*:\s*[^;]+;', html_string)
    used_vars = set(re.findall(r'var\(--[\w-]+\)', html_string))
    used_vars = {re.sub(r'var\(([^)]+)\)', r'\1', variable) for variable in used_vars}
    unused_var_definitions = [var_def for var_def in var_definitions if var_def.split(':')[0] not in used_vars]
    for unused_var_def in unused_var_definitions:
        html_string = html_string.replace(unused_var_def, '')

    # Remove characters ignored by modern browsers
    html_string = html_string.replace('\n', '')
    html_string = html_string.replace('\t', '')
    html_string = html_string.replace('\r', '')
    html_string = html_string.replace('\ufeff', '')
    html_string = html_string.replace('\u200b', '')

    # Substitute CSS class names, IDs, and variables with one-letter equivalents
    css_variables = set(re.findall(r'--[\w-]+', html_string))
    for i, css_variable in enumerate(css_variables):
        html_string = html_string.replace(css_variable, f'--v{i}')

    # Search fir classes and ids in the css definitions of the file and substitute all usages (FIXME)
    """css_definitions = ''.join(re.findall(r'<style>(.*?)</style>', html_string, flags=re.DOTALL))
    css_classes = set(re.findall(r'\.([\w-]+)', css_definitions))
    css_ids = set(re.findall(r'#([\w-]+)[{>,:]', css_definitions))
    for i, css_class in enumerate(css_classes):
        html_string = re.sub(rf'class\s*=\s*"(.*)\b{css_class}\b(.*)"', rf'class="\1c{i}\2"', html_string)
        html_string = re.sub(fr'\s*\.{css_class}\s*{r"([{>,:])"}\s*', rf'#c{i}\1', html_string)
    for i, css_id in enumerate(css_ids):
        html_string = re.sub(rf'id\s*=\s*"{css_id}"', rf'id="i{i}"', html_string)
        html_string = re.sub(fr'\s*#{css_id}\s*{r"([{>,:])"}\s*', rf'#i{i}\1', html_string)
    if html_string.find('id="projects"') >= 0:
        print(html_string)
        print(css_ids)"""

    # remove redundant spaces
    html_string = re.sub(r'\s+', ' ', html_string)


    os.makedirs(os.path.dirname(save_path), exist_ok=True)
    print(f' > {save_path}')
    with open(save_path, 'w', encoding='utf-8') as file:
        file.write(html_string)


if __name__ == '__main__':
    main()