from playwright.sync_api import sync_playwright
import os
import html2text

RESULT_KEY = '<ul class="search-results active">'
RESULT_END_KEY = '</ul>'
LEN_KEY = len(RESULT_KEY)
DOC_BASE_URL = "file:///Users/syc/Projects/verus/source/doc/vstd/index.html?search="
DOC_BASE_PATH = "file:///Users/syc/Projects/verus/source/doc/vstd"

def fetch_html(url: str) -> str:
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        page.goto(url, wait_until="networkidle")  
        content = page.content()
        browser.close()
    return content


def _search(search_key: str) -> str:
    content = fetch_html(DOC_BASE_URL + search_key)
    pos = content.find(RESULT_KEY)
    pos2 = content.find(RESULT_END_KEY, pos)
    content = content[pos+LEN_KEY:pos2].replace('<a class', '\n<a class')
    return content

import re
def cleanup_html(html: str) -> str:
    clean = html
    clean = re.sub(
        r'<rustdoc-toolbar\b[^>]*>.*?</rustdoc-toolbar>', 
        '', 
        clean, 
        flags=re.DOTALL
    )
    clean = re.sub(
        r'<button\b[^>]*>.*?</button>', 
        '', 
        clean, 
        flags=re.DOTALL
    )
    return clean

def clean_md(md: str) -> str:
    clean = md
    clean = re.sub(r'\[Source\]\([^)]*\)', '', clean)
    clean = re.sub(r'\[([^\]]+)\]\([^)]*\)', r'\1', clean)
    clean = '\n'.join([x.strip() for x in clean.split('\n')])
    clean = re.sub(r'\n{3,}', '\n\n', clean)
    return clean


def search_doc(search_key: str) -> str:
    content = _search(search_key)
    href_pos = content.find('href="') + 6
    href_end_pos = content.find('"', href_pos)
    path = content[href_pos:href_end_pos].replace('../vstd', DOC_BASE_PATH)
    html = fetch_html(path)
    content_pos = html.find('class="content"')
    gt_pos = html.find('>', content_pos)
    content_end_pos = html.rfind('</section>', gt_pos)
    #print(html[gt_pos+1:content_end_pos])
    main_html = cleanup_html(html[gt_pos+1:content_end_pos])
    #print(main_html)
    #print('=' * 50)
    return clean_md(html2text.html2text(main_html).replace('§', ''))

print(search_doc("Resource"))

