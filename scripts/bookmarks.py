#!/usr/bin/env python3
"""浏览器书签管理工具"""

import json
import sys
import argparse
from pathlib import Path
from collections import defaultdict
from urllib.parse import urlparse


def find_bookmarks_file(browser=None, profile=None):
    home = Path.home()
    candidates = []

    if browser in (None, 'edge'):
        base = home / 'Library/Application Support/Microsoft Edge'
        if base.exists():
            for p in sorted(base.glob('*/Bookmarks')):
                candidates.append(('edge', p))

    if browser in (None, 'chrome'):
        base = home / 'Library/Application Support/Google/Chrome'
        if base.exists():
            for p in sorted(base.glob('*/Bookmarks')):
                candidates.append(('chrome', p))

    if not candidates:
        return None, None

    if profile:
        filtered = [(b, p) for b, p in candidates if f'Profile {profile}' in str(p)]
        if filtered:
            candidates = filtered

    return candidates[0]


def load_bookmarks(path):
    with open(path, 'r', encoding='utf-8') as f:
        return json.load(f)


def extract_all(node, path=''):
    results = []
    name = node.get('name', '')
    if node.get('type') == 'url':
        results.append({'name': name, 'url': node.get('url', ''), 'path': path})
    elif 'children' in node:
        new_path = f'{path}/{name}' if name else path
        for child in node.get('children', []):
            results.extend(extract_all(child, new_path))
    return results


def collect_folder_stats(node, path='', stats=None):
    if stats is None:
        stats = []
    name = node.get('name', '')
    children = node.get('children', [])
    urls = [c for c in children if c.get('type') == 'url']
    folders = [c for c in children if 'children' in c]
    new_path = f'{path}/{name}' if name else path
    if name:
        stats.append({
            'path': new_path,
            'name': name,
            'url_count': len(urls),
            'folder_count': len(folders),
        })
    for child in folders:
        collect_folder_stats(child, new_path, stats)
    return stats


def print_tree(node, depth=0):
    name = node.get('name', '')
    children = node.get('children', [])
    urls = [c for c in children if c.get('type') == 'url']
    folders = [c for c in children if 'children' in c]
    if depth > 0 and name:
        indent = '  ' * depth
        print(f'{indent}📁 {name}  ({len(urls)} 书签, {len(folders)} 子文件夹)')
    for folder in folders:
        print_tree(folder, depth + 1)


def cmd_structure(args, path):
    data = load_bookmarks(path)
    roots = data.get('roots', {})
    all_bm = []
    for node in roots.values():
        all_bm.extend(extract_all(node))
    print(f'总书签数: {len(all_bm)}\n')
    print_tree(roots.get('bookmark_bar', {}))


def cmd_dupes(args, path):
    data = load_bookmarks(path)
    roots = data.get('roots', {})
    all_bm = []
    for node in roots.values():
        all_bm.extend(extract_all(node))

    # URL 重复
    by_url = defaultdict(list)
    for bm in all_bm:
        by_url[bm['url'].rstrip('/')].append(bm)
    url_dupes = {u: v for u, v in by_url.items() if len(v) > 1}

    print('=== URL 完全重复 ===')
    if url_dupes:
        for url, bms in sorted(url_dupes.items(), key=lambda x: -len(x[1])):
            print(f'[{len(bms)}次] {bms[0]["name"][:60]}')
            print(f'  URL: {url[:80]}')
            for bm in bms:
                print(f'  📁 {bm["path"]}')
            print()
    else:
        print('无 URL 完全重复的书签\n')

    # 名称重复
    by_name = defaultdict(list)
    for bm in all_bm:
        n = bm['name'].strip()
        if n:
            by_name[n].append(bm)
    name_dupes = {n: v for n, v in by_name.items() if len(v) > 1}

    print('=== 名称重复（URL 不同）===')
    if name_dupes:
        for name, bms in sorted(name_dupes.items(), key=lambda x: -len(x[1]))[:20]:
            print(f'[{len(bms)}次] {name[:60]}')
            for bm in bms:
                print(f'  📁 {bm["path"]}')
                print(f'     {bm["url"][:70]}')
            print()
    else:
        print('无名称重复的书签\n')


def cmd_analyze(args, path):
    data = load_bookmarks(path)
    roots = data.get('roots', {})
    all_bm = []
    folder_stats = []
    for node in roots.values():
        all_bm.extend(extract_all(node))
        collect_folder_stats(node, stats=folder_stats)

    print(f'总书签数: {len(all_bm)}，总文件夹数: {len(folder_stats)}\n')

    overloaded = [f for f in folder_stats if f['url_count'] > 30]
    if overloaded:
        print('=== 书签过多（>30）建议细分 ===')
        for f in sorted(overloaded, key=lambda x: -x['url_count']):
            print(f'  {f["url_count"]:4d} 个  {f["path"]}')
        print()

    has_both = [f for f in folder_stats if f['url_count'] > 0 and f['folder_count'] > 0]
    if has_both:
        print('=== 父文件夹有散落书签（建议归入子文件夹）===')
        for f in sorted(has_both, key=lambda x: -x['url_count']):
            print(f'  {f["url_count"]:3d} 个散落  📁 {f["path"]}')
        print()

    sparse = [f for f in folder_stats if 0 < f['url_count'] <= 3 and f['folder_count'] == 0]
    if sparse:
        print('=== 书签过少（≤3）可考虑合并 ===')
        for f in sorted(sparse, key=lambda x: x['url_count']):
            print(f'  {f["url_count"]:3d} 个  {f["path"]}')
        print()

    by_domain = defaultdict(int)
    for bm in all_bm:
        domain = urlparse(bm['url']).netloc
        if domain:
            by_domain[domain] += 1
    print('=== 域名分布 Top 15 ===')
    for domain, count in sorted(by_domain.items(), key=lambda x: -x[1])[:15]:
        print(f'  {count:4d}  {domain}')


def cmd_search(args, path):
    query = ' '.join(args.query).lower()
    data = load_bookmarks(path)
    roots = data.get('roots', {})
    all_bm = []
    for node in roots.values():
        all_bm.extend(extract_all(node))

    results = [bm for bm in all_bm if query in bm['name'].lower() or query in bm['url'].lower()]
    print(f'搜索 "{query}"，共 {len(results)} 条结果:\n')
    for bm in results[:50]:
        print(f'📁 {bm["path"]}')
        print(f'   {bm["name"][:70]}')
        print(f'   {bm["url"][:80]}')
        print()
    if len(results) > 50:
        print(f'... 还有 {len(results) - 50} 条，请缩小搜索范围')


def main():
    parser = argparse.ArgumentParser(description='浏览器书签管理工具')
    parser.add_argument('--browser', choices=['chrome', 'edge'])
    parser.add_argument('--profile', help='Profile 编号，如 1')
    sub = parser.add_subparsers(dest='cmd')
    sub.add_parser('structure', help='展示文件夹树')
    sub.add_parser('dupes', help='查找重复书签')
    sub.add_parser('analyze', help='分析结构问题')
    sp = sub.add_parser('search', help='搜索书签')
    sp.add_argument('query', nargs='+')

    args = parser.parse_args()

    browser, bm_path = find_bookmarks_file(args.browser, args.profile)
    if not bm_path:
        print('未找到书签文件', file=sys.stderr)
        sys.exit(1)

    print(f'[{browser}  {bm_path}]\n')

    cmd = args.cmd or 'structure'
    {'structure': cmd_structure, 'dupes': cmd_dupes, 'analyze': cmd_analyze, 'search': cmd_search}[cmd](args, bm_path)


if __name__ == '__main__':
    main()
