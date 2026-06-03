import os

for root, _, files in os.walk('.'):
    if '.git' in root or 'node_modules' in root or '.docusaurus' in root:
        continue
    for file in files:
        if file.endswith(('.png', '.jpg', '.ico', '.svg', '.json', '.lock')):
            continue
        path = os.path.join(root, file)
        try:
            with open(path, 'r', encoding='utf-8') as f:
                c = f.read()
            if 'AMUD-Dashboard' in c:
                c = c.replace('AMUD-Dashboard', 'AMUD-Dashboard')
                with open(path, 'w', encoding='utf-8') as f:
                    f.write(c)
                print(f"Updated {path}")
        except Exception as e:
            pass
