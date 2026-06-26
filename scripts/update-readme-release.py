#!/usr/bin/env python3
"""Update localized README 'What's new' blocks for v1.5.6.3."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

BLOCKS: dict[str, str] = {
    "README.md": """### What's new in v1.5.6.3

- **37 bundled themes** — visual **Theme Gallery** in Settings → Appearance (preview CSS + wallpaper, then Save)
- **18 new theme packs** — Nature, Terminal, Feminine, Variety; each with its own vendored Unsplash/Pexels wallpaper
- **Guest dashboard** — compact cards (icon, name, online/offline only)
- **RSS settings** — add-feed modal + category table layout fixes
- **Integration cards** — filled 6-cell stats grid and 30s live refresh (v1.5.6.2)

Full history: **[Changelog](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Release status (audit 2026-06-26)

After manual validation in a clean Proxmox test container, these releases are currently kept as known-good baselines:

- `v1.0.0`
- `v1.3.6`
- `v1.3.7`
- `v1.4.1.0`
- `v1.5.5.3`
- `v1.5.5.6`
- `v1.5.5.9`
- `v1.5.6.0`
- `v1.5.6.2`
- `v1.5.6.3` (current latest recommended)

Broken tags identified during audit were removed from GitHub releases/tags and should not be used. **Do not use `v1.5.5.4` or `v1.5.6.1`.**""",
    "readmes/README.es.md": """### Novedades en v1.5.6.3

- **37 temas incluidos** — **Galería de temas** visual en Ajustes → Apariencia (vista previa de CSS + fondo, luego Guardar)
- **18 paquetes nuevos** — Naturaleza, Terminal, Femenino, Variedad; cada uno con fondo Unsplash/Pexels incluido
- **Panel invitado** — tarjetas compactas (icono, nombre, en línea/desconectado)
- **Ajustes RSS** — modal de nuevo feed y tabla de categorías corregida
- **Tarjetas de integración** — cuadrícula de 6 celdas y actualización cada 30 s (v1.5.6.2)

Historial completo: **[Registro de cambios](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Estado de versiones (auditoría 2026-06-26)

- `v1.5.6.3` (última recomendada actualmente) · también validadas: `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

No uses `v1.5.5.4` ni `v1.5.6.1` (retirada).""",
    "readmes/README.pt.md": """### Novidades na v1.5.6.3

- **37 temas incluídos** — **Galeria de temas** visual em Configurações → Aparência (pré-visualizar CSS + papel de parede, depois Salvar)
- **18 novos pacotes** — Natureza, Terminal, Feminino, Variedade; cada um com wallpaper Unsplash/Pexels incluído
- **Painel convidado** — cartões compactos (ícone, nome, online/offline)
- **RSS** — modal de novo feed e tabela de categorias corrigida
- **Cartões de integração** — grelha de 6 células e atualização a cada 30 s (v1.5.6.2)

Histórico completo: **[Changelog](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Estado da versão (auditoria 2026-06-26)

- `v1.5.6.3` (recomendada) · também validadas: `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

Não use `v1.5.5.4` nem `v1.5.6.1` (retirada).""",
    "readmes/README.fr.md": """### Nouveautés de la v1.5.6.3

- **37 thèmes inclus** — **Galerie de thèmes** visuelle dans Paramètres → Apparence (aperçu CSS + fond, puis Enregistrer)
- **18 nouveaux packs** — Nature, Terminal, Féminin, Variété ; fond d'écran Unsplash/Pexels inclus pour chacun
- **Tableau invité** — cartes compactes (icône, nom, en ligne/hors ligne)
- **RSS** — modal d'ajout de flux et tableau des catégories corrigé
- **Cartes d'intégration** — grille 6 cellules et rafraîchissement 30 s (v1.5.6.2)

Historique complet : **[Journal des modifications](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### État des versions (audit 2026-06-26)

- `v1.5.6.3` (dernière recommandée) · également validées : `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

N'utilisez pas `v1.5.5.4` ni `v1.5.6.1` (retirée).""",
    "readmes/README.de.md": """### Was ist neu in v1.5.6.3

- **37 mitgelieferte Themes** — visuelle **Theme-Galerie** unter Einstellungen → Erscheinungsbild (CSS + Hintergrundbild vorab, dann Speichern)
- **18 neue Pakete** — Natur, Terminal, Feminine, Variety; jeweils eigenes Unsplash/Pexels-Hintergrundbild offline
- **Gast-Dashboard** — kompakte Karten (Icon, Name, Online/Offline)
- **RSS-Einstellungen** — Feed-Modal und Kategorietabelle behoben
- **Integrationskarten** — 6-Zellen-Raster, 30-Sekunden-Aktualisierung (v1.5.6.2)

Vollständiger Verlauf: **[Changelog](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Release-Status (Audit 2026-06-26)

- `v1.5.6.3` (aktuell empfohlen) · außerdem validiert: `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

**Nicht** `v1.5.5.4` oder `v1.5.6.1` (zurückgezogen) verwenden.""",
    "readmes/README.it.md": """### Novità in v1.5.6.3

- **37 temi inclusi** — **Galleria temi** visuale in Impostazioni → Aspetto (anteprima CSS + sfondo, poi Salva)
- **18 nuovi pacchetti** — Natura, Terminal, Femminile, Varietà; sfondo Unsplash/Pexels per ciascuno
- **Dashboard ospite** — schede compatte (icona, nome, online/offline)
- **RSS** — modale nuovo feed e tabella categorie corretta
- **Schede integrazione** — griglia 6 celle, aggiornamento ogni 30 s (v1.5.6.2)

Cronologia completa: **[Changelog](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Stato release (audit 2026-06-26)

- `v1.5.6.3` (ultima consigliata) · validate anche: `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

Non usare `v1.5.5.4` né `v1.5.6.1` (ritirata).""",
    "readmes/README.ru.md": """### Что нового в v1.5.6.3

- **37 встроенных тем** — визуальная **галерея тем** в Настройки → Внешний вид (предпросмотр CSS + обои, затем Сохранить)
- **18 новых наборов** — Природа, Терминал, Женские, Разнообразие; у каждой свои обои Unsplash/Pexels
- **Гостевая панель** — компактные карточки (иконка, имя, онлайн/офлайн)
- **RSS** — модальное окно добавления ленты и таблица категорий
- **Интеграции** — сетка из 6 ячеек, обновление каждые 30 с (v1.5.6.2)

Полная история: **[Журнал изменений](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### Статус релизов (аудит 2026-06-26)

- `v1.5.6.3` (рекомендуется) · также проверены: `v1.5.6.2`, `v1.5.6.0`, `v1.5.5.9`, `v1.5.5.6`, `v1.5.5.3`, `v1.4.1.0`, `v1.3.7`, `v1.3.6`, `v1.0.0`

Не используйте `v1.5.5.4` и `v1.5.6.1` (отозвана).""",
    "readmes/README.zh.md": """### v1.5.6.3 新特性

- **37 款内置主题** — 设置 → 外观中的可视化**主题画廊**（预览 CSS + 壁纸后保存）
- **18 个新主题包** — 自然、终端、柔美、多样；各含离线 Unsplash/Pexels 壁纸
- **访客面板** — 紧凑卡片（图标、名称、在线/离线）
- **RSS 设置** — 添加订阅弹窗与分类表布局修复
- **集成卡片** — 6 格统计与 30 秒刷新（v1.5.6.2）

完整历史：**[更新日志](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### 版本状态（审计 2026-06-26）

- `v1.5.6.3`（当前推荐）· 亦已验证：`v1.5.6.2`、`v1.5.6.0`、`v1.5.5.9` 等

请勿使用 `v1.5.5.4` 或 `v1.5.6.1`（已撤回）。""",
    "readmes/README.ja.md": """### v1.5.6.3 の新機能

- **37 のバンドルテーマ** — 設定 → 外観の**テーマギャラリー**（CSS + 壁紙をプレビューして保存）
- **18 の新パック** — 自然・ターミナル・フェミニン・バラエティ；各テーマ専用の Unsplash/Pexels 壁紙
- **ゲスト画面** — コンパクトカード（アイコン・名前・オンライン/オフライン）
- **RSS 設定** — フィード追加モーダルとカテゴリ表の修正
- **統合カード** — 6 セルグリッド・30 秒更新（v1.5.6.2）

履歴: **[変更履歴](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### リリース状況（監査 2026-06-26）

- `v1.5.6.3`（推奨）· `v1.5.5.4` と `v1.5.6.1`（撤回）は使用しないこと""",
    "readmes/README.hi.md": """### v1.5.6.3 में नया

- **37 बंडल थीम** — सेटिंग्स → दिखावट में विज़ुअल **थीम गैलरी** (CSS + वॉलपेपर पूर्वावलोकन, फिर सेव)
- **18 नए पैक** — प्रकृति, टर्मिनल, फेमिनिन, विविधता; प्रत्येक का अपना Unsplash/Pexels वॉलपेपर
- **गेस्ट डैशबोर्ड** — कॉम्पैक्ट कार्ड (आइकन, नाम, ऑनलाइन/ऑफलाइन)
- **RSS सेटिंग्स** — फ़ीड मोडल और श्रेणी तालिका सुधार
- **इंटीग्रेशन कार्ड** — 6-सेल ग्रिड, 30 सेकंड रिफ्रेश (v1.5.6.2)

पूरा इतिहास: **[चेंजलॉग](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### रिलीज़ स्थिति (ऑडिट 2026-06-26)

- `v1.5.6.3` (अनुशंसित) · `v1.5.5.4` या `v1.5.6.1` (वापस ली गई) न उपयोग करें""",
    "readmes/README.ko.md": """### v1.5.6.3 새로운 기능

- **37개 번들 테마** — 설정 → 모양의 **테마 갤러리**(CSS + 배경화면 미리보기 후 저장)
- **18개 신규 팩** — 자연, 터미널, 페미닌, 버라이어티; 테마별 Unsplash/Pexels 배경화면 포함
- **게스트 대시보드** — 컴팩트 카드(아이콘, 이름, 온라인/오프라인)
- **RSS 설정** — 피드 추가 모달 및 카테고리 표 수정
- **통합 카드** — 6칸 그리드, 30초 새로고침(v1.5.6.2)

전체 기록: **[변경 로그](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### 릴리스 상태(감사 2026-06-26)

- `v1.5.6.3`(권장) · `v1.5.5.4`, `v1.5.6.1`(철회) 사용 금지""",
    "readmes/README.ar.md": """### الجديد في v1.5.6.3

- **37 سمة مدمجة** — **معرض السمات** المرئي في الإعدادات → المظهر (معاينة CSS + الخلفية ثم حفظ)
- **18 حزمة جديدة** — الطبيعة، الطرفية، أنثوية، متنوعة؛ خلفية Unsplash/Pexels لكل سمة
- **لوحة الضيف** — بطاقات مدمجة (أيقونة، اسم، متصل/غير متصل)
- **إعدادات RSS** — نافذة إضافة التغذية وجدول الفئات
- **بطاقات التكامل** — شبكة 6 خلايا وتحديث كل 30 ثانية (v1.5.6.2)

السجل الكامل: **[سجل التغييرات](https://boubli.github.io/AMUD-Dashboard/docs/changelog)**

### حالة الإصدار (تدقيق 2026-06-26)

- `v1.5.6.3` (موصى به) · لا تستخدم `v1.5.5.4` أو `v1.5.6.1` (مسحوبة)""",
}

PATTERN = re.compile(r"### .+?\n\n!\[AMUD Dashboard UI\]", re.DOTALL)


def main() -> None:
    for rel, block in BLOCKS.items():
        path = ROOT / rel
        text = path.read_text(encoding="utf-8")
        if not PATTERN.search(text):
            print(f"SKIP (no match): {rel}")
            continue
        new_text = PATTERN.sub(block + "\n\n![AMUD Dashboard UI]", text, count=1)
        path.write_text(new_text, encoding="utf-8")
        print(f"OK {rel}")


if __name__ == "__main__":
    main()
