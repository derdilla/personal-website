import os
from datetime import datetime

def main():
    print('\033[94mBuilding sitemap...\033[0m')

    posts = ''
    for file_name in os.listdir('blog/posts'):
        name = file_name.split('.')
        if name[1] != 'blog':
            continue
        posts += f'''<url>
            <loc>https://derdilla.com/blog/{name[0]}</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime(f'blog/posts/{file_name}')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>yearly</changefreq>
            <priority>0.9</priority>
        </url>'''

    map = f'''<?xml version='1.0' encoding='UTF-8'?>
    <urlset xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:schemaLocation="http://www.sitemaps.org/schemas/sitemap/0.9 http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd"
        xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url>
            <loc>https://derdilla.com/</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime('index.html')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>yearly</changefreq>
            <priority>0.3</priority>
        </url>
        <url>
            <loc>https://derdilla.com/about</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime('about/index.html')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>yearly</changefreq>
            <priority>0.5</priority>
        </url>
        <url>
            <loc>https://derdilla.com/faq</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime('faq/index.html')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>monthly</changefreq>
            <priority>0.6</priority>
        </url>
        <url>
            <loc>https://derdilla.com/projects</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime('projects/index.html')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>monthly</changefreq>
            <priority>0.8</priority>
        </url>
        <url>
            <loc>https://derdilla.com/blog</loc>
            <lastmod>{datetime.fromtimestamp(os.path.getmtime('blog')).astimezone().replace(microsecond=0).isoformat()}</lastmod>
            <changefreq>daily</changefreq>
            <priority>1.0</priority>
        </url>
        {posts}
    </urlset>'''
    # timestamps from css files not required for indexing

    print('\033[94mWriting sitemap to out/sitemap.xml...\033[0m')
    with open('out/sitemap.xml', 'w') as f:
        f.write(map)



if  __name__ == '__main__':
    main()