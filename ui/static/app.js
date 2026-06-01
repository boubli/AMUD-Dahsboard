document.addEventListener('DOMContentLoaded', () => {
    if (window.lucide) {
        lucide.createIcons();
    }
});

document.addEventListener('htmx:afterSwap', () => {
    if (window.lucide) {
        lucide.createIcons();
    }
});

document.addEventListener('alpine:init', () => {
    Alpine.data('dashboardShell', () => ({
        theme: 'aurora',
        themeClass: 'theme-aurora',
        appName: 'AMUD Dashboard',
        tagline: 'High-Performance Intelligent Home Lab Cockpit',
        username: 'admin',
        searchEnabled: true,
        showGreeting: true,
        showClock: true,
        drawerOpen: false,
        searchQuery: '',
        greeting: 'Good evening',
        localClock: '',
        init() {
            const dataset = document.body.dataset;
            this.appName = dataset.appName || this.appName;
            this.tagline = dataset.tagline || this.tagline;
            this.username = dataset.username || this.username;
            this.searchEnabled = dataset.searchEnabled !== 'false';
            this.showGreeting = dataset.showGreeting !== 'false';
            this.showClock = dataset.showClock !== 'false';
            this.theme = localStorage.getItem('amud.theme') || dataset.backgroundTheme || this.theme;
            this.applyTheme();
            this.updateGreeting();
            this.updateClock();
            window.setInterval(() => this.updateClock(), 60000);
            window.setInterval(() => this.updateGreeting(), 60000);
        },
        applyTheme() {
            this.themeClass = `theme-${this.theme}`;
            localStorage.setItem('amud.theme', this.theme);
        },
        setTheme(themeName) {
            this.theme = themeName;
            this.applyTheme();
        },
        updateGreeting() {
            const hour = new Date().getHours();
            if (hour < 12) {
                this.greeting = 'Good morning';
            } else if (hour < 18) {
                this.greeting = 'Good afternoon';
            } else if (hour < 22) {
                this.greeting = 'Good evening';
            } else {
                this.greeting = 'Good night';
            }
        },
        updateClock() {
            const now = new Date();
            this.localClock = new Intl.DateTimeFormat('en-GB', {
                hour: '2-digit',
                minute: '2-digit',
                second: undefined,
            }).format(now);
        },
        focusSearch() {
            if (!this.searchEnabled) {
                this.drawerOpen = true;
                return;
            }

            this.$nextTick(() => {
                if (this.$refs.searchInput) {
                    this.$refs.searchInput.focus();
                    this.$refs.searchInput.select();
                }
            });
        },
    }));
});
