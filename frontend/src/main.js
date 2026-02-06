/**
 * Net-Relay Dashboard
 * Frontend JavaScript for monitoring proxy connections and managing settings
 */

const API_BASE = '/api';
const REFRESH_INTERVAL = 2000; // 2 seconds

class Dashboard {
    constructor() {
        this.elements = {
            status: document.getElementById('status'),
            activeConnections: document.getElementById('active-connections'),
            totalConnections: document.getElementById('total-connections'),
            bytesSent: document.getElementById('bytes-sent'),
            bytesReceived: document.getElementById('bytes-received'),
            uptime: document.getElementById('uptime'),
            activeTbody: document.getElementById('active-tbody'),
            historyTbody: document.getElementById('history-tbody'),
            version: document.getElementById('version'),
        };
        
        this.isConnected = false;
        this.accessControl = null;
        this.init();
    }

    async init() {
        this.setupTabs();
        this.setupSettingsHandlers();
        
        await this.checkHealth();
        await this.refresh();
        await this.loadHistory();
        await this.loadAccessControl();
        
        // Start periodic refresh
        setInterval(() => this.refresh(), REFRESH_INTERVAL);
        setInterval(() => this.loadHistory(), REFRESH_INTERVAL * 5);
    }

    // ==================== Tab Navigation ====================
    
    setupTabs() {
        const tabBtns = document.querySelectorAll('.tab-btn');
        tabBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                const tabName = btn.dataset.tab;
                this.switchTab(tabName);
            });
        });
    }

    switchTab(tabName) {
        // Update buttons
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.tab === tabName);
        });
        
        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.toggle('active', content.id === `${tabName}-tab`);
        });

        // Refresh settings when switching to settings tab
        if (tabName === 'settings') {
            this.loadAccessControl();
        }
    }

    // ==================== Settings Handlers ====================

    setupSettingsHandlers() {
        // IP Blacklist
        document.getElementById('add-blacklist-btn')?.addEventListener('click', () => {
            const input = document.getElementById('blacklist-ip-input');
            if (input.value.trim()) {
                this.addIpBlacklist(input.value.trim());
                input.value = '';
            }
        });

        // IP Whitelist
        document.getElementById('add-whitelist-btn')?.addEventListener('click', () => {
            const input = document.getElementById('whitelist-ip-input');
            if (input.value.trim()) {
                this.addIpWhitelist(input.value.trim());
                input.value = '';
            }
        });

        // Add Rule
        document.getElementById('add-rule-btn')?.addEventListener('click', () => {
            const name = document.getElementById('rule-name').value.trim();
            const domain = document.getElementById('rule-domain').value.trim();
            const path = document.getElementById('rule-path').value.trim() || null;
            const action = document.getElementById('rule-action').value;

            if (domain) {
                this.addRule({ name, domain, path, action, enabled: true });
                document.getElementById('rule-name').value = '';
                document.getElementById('rule-domain').value = '';
                document.getElementById('rule-path').value = '';
            }
        });
    }

    // ==================== Access Control API ====================

    async loadAccessControl() {
        try {
            const response = await fetch(`${API_BASE}/config/access-control`);
            const data = await response.json();
            
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to load access control:', error);
        }
    }

    renderAccessControl() {
        if (!this.accessControl) return;

        // Render IP Blacklist
        this.renderIpList('ip-blacklist', this.accessControl.ip_blacklist, 'blacklist');
        
        // Render IP Whitelist
        this.renderIpList('ip-whitelist', this.accessControl.ip_whitelist, 'whitelist');
        
        // Render mode
        const modeDisplay = document.getElementById('mode-display');
        const modeDesc = document.getElementById('mode-desc');
        if (this.accessControl.allow_by_default) {
            modeDisplay.textContent = 'Blacklist';
            modeDesc.textContent = 'All domains allowed except blocked ones';
        } else {
            modeDisplay.textContent = 'Whitelist';
            modeDesc.textContent = 'All domains blocked except allowed ones';
        }

        // Render rules
        this.renderRules();
    }

    renderIpList(containerId, ips, type) {
        const container = document.getElementById(containerId);
        if (!container) return;

        if (ips.length === 0) {
            container.innerHTML = '<span class="empty-text">No IPs configured</span>';
            return;
        }

        container.innerHTML = ips.map(ip => `
            <span class="tag ${type}">
                ${this.escapeHtml(ip)}
                <button class="tag-remove" data-ip="${this.escapeHtml(ip)}" data-type="${type}">&times;</button>
            </span>
        `).join('');

        // Add remove handlers
        container.querySelectorAll('.tag-remove').forEach(btn => {
            btn.addEventListener('click', () => {
                const ip = btn.dataset.ip;
                const type = btn.dataset.type;
                if (type === 'blacklist') {
                    this.removeIpBlacklist(ip);
                } else {
                    this.removeIpWhitelist(ip);
                }
            });
        });
    }

    renderRules() {
        const tbody = document.getElementById('rules-tbody');
        if (!tbody) return;

        const rules = this.accessControl.rules || [];
        
        if (rules.length === 0) {
            tbody.innerHTML = '<tr class="empty-row"><td colspan="5">No rules configured</td></tr>';
            return;
        }

        tbody.innerHTML = rules.map((rule, index) => `
            <tr>
                <td>${this.escapeHtml(rule.name || '-')}</td>
                <td><code>${this.escapeHtml(rule.domain)}</code></td>
                <td><code>${rule.path ? this.escapeHtml(rule.path) : '*'}</code></td>
                <td><span class="action-badge ${rule.action}">${rule.action}</span></td>
                <td>
                    <button class="btn btn-sm btn-danger remove-rule" data-index="${index}">Remove</button>
                </td>
            </tr>
        `).join('');

        // Add remove handlers
        tbody.querySelectorAll('.remove-rule').forEach(btn => {
            btn.addEventListener('click', () => {
                this.removeRule(parseInt(btn.dataset.index));
            });
        });
    }

    async addIpBlacklist(ip) {
        try {
            const response = await fetch(`${API_BASE}/config/ip/blacklist`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip })
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to add IP to blacklist:', error);
        }
    }

    async removeIpBlacklist(ip) {
        try {
            const response = await fetch(`${API_BASE}/config/ip/blacklist`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip })
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to remove IP from blacklist:', error);
        }
    }

    async addIpWhitelist(ip) {
        try {
            const response = await fetch(`${API_BASE}/config/ip/whitelist`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip })
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to add IP to whitelist:', error);
        }
    }

    async removeIpWhitelist(ip) {
        try {
            const response = await fetch(`${API_BASE}/config/ip/whitelist`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip })
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to remove IP from whitelist:', error);
        }
    }

    async addRule(rule) {
        try {
            const response = await fetch(`${API_BASE}/config/rules`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(rule)
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to add rule:', error);
        }
    }

    async removeRule(index) {
        try {
            const response = await fetch(`${API_BASE}/config/rules`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ index })
            });
            const data = await response.json();
            if (data.success) {
                this.accessControl = data.data;
                this.renderAccessControl();
            }
        } catch (error) {
            console.error('Failed to remove rule:', error);
        }
    }

    // ==================== Dashboard Stats ====================

    async checkHealth() {
        try {
            const response = await fetch(`${API_BASE}/health`);
            const data = await response.json();
            
            if (data.success) {
                this.setConnected(true);
                this.elements.version.textContent = data.data.version;
            } else {
                this.setConnected(false);
            }
        } catch (error) {
            console.error('Health check failed:', error);
            this.setConnected(false);
        }
    }

    setConnected(connected) {
        this.isConnected = connected;
        const statusEl = this.elements.status;
        
        if (connected) {
            statusEl.textContent = 'Connected';
            statusEl.className = 'status connected';
        } else {
            statusEl.textContent = 'Disconnected';
            statusEl.className = 'status error';
        }
    }

    async refresh() {
        try {
            const response = await fetch(`${API_BASE}/stats`);
            const data = await response.json();
            
            if (data.success) {
                this.setConnected(true);
                this.updateStats(data.data.aggregated);
                this.updateActiveConnections(data.data.active_connections);
            } else {
                this.setConnected(false);
            }
        } catch (error) {
            console.error('Failed to fetch stats:', error);
            this.setConnected(false);
        }
    }

    updateStats(stats) {
        this.elements.activeConnections.textContent = stats.active_connections.toLocaleString();
        this.elements.totalConnections.textContent = stats.total_connections.toLocaleString();
        this.elements.bytesSent.textContent = this.formatBytes(stats.total_bytes_sent);
        this.elements.bytesReceived.textContent = this.formatBytes(stats.total_bytes_received);
        this.elements.uptime.textContent = this.formatDuration(stats.uptime_secs);
    }

    updateActiveConnections(connections) {
        const tbody = this.elements.activeTbody;
        
        if (connections.length === 0) {
            tbody.innerHTML = '<tr class="empty-row"><td colspan="6">No active connections</td></tr>';
            return;
        }

        tbody.innerHTML = connections.map(conn => `
            <tr>
                <td><span class="protocol-badge ${conn.protocol}">${conn.protocol}</span></td>
                <td>${this.escapeHtml(conn.client_addr)}</td>
                <td>${this.escapeHtml(conn.target_addr)}:${conn.target_port}</td>
                <td>${this.formatDuration(this.calculateDuration(conn.connected_at))}</td>
                <td>${this.formatBytes(conn.bytes_sent)}</td>
                <td>${this.formatBytes(conn.bytes_received)}</td>
            </tr>
        `).join('');
    }

    async loadHistory() {
        try {
            const response = await fetch(`${API_BASE}/history?limit=50`);
            const data = await response.json();
            
            if (data.success) {
                this.updateHistory(data.data);
            }
        } catch (error) {
            console.error('Failed to fetch history:', error);
        }
    }

    updateHistory(history) {
        const tbody = this.elements.historyTbody;
        
        if (history.length === 0) {
            tbody.innerHTML = '<tr class="empty-row"><td colspan="7">No connection history</td></tr>';
            return;
        }

        tbody.innerHTML = history.map(item => {
            const conn = item;
            return `
                <tr>
                    <td><span class="protocol-badge ${conn.protocol}">${conn.protocol}</span></td>
                    <td>${this.escapeHtml(conn.client_addr)}</td>
                    <td>${this.escapeHtml(conn.target_addr)}:${conn.target_port}</td>
                    <td>${this.formatDuration(this.calculateConnectionDuration(conn))}</td>
                    <td>${this.formatBytes(conn.bytes_sent)}</td>
                    <td>${this.formatBytes(conn.bytes_received)}</td>
                    <td>${this.formatTime(conn.closed_at)}</td>
                </tr>
            `;
        }).join('');
    }

    // ==================== Utilities ====================

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    formatDuration(seconds) {
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        return `${hours}h ${minutes}m`;
    }

    formatTime(isoString) {
        if (!isoString) return '-';
        const date = new Date(isoString);
        return date.toLocaleTimeString();
    }

    calculateDuration(startTime) {
        const start = new Date(startTime);
        const now = new Date();
        return Math.floor((now - start) / 1000);
    }

    calculateConnectionDuration(conn) {
        const start = new Date(conn.connected_at);
        const end = conn.closed_at ? new Date(conn.closed_at) : new Date();
        return Math.floor((end - start) / 1000);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new Dashboard();
});
