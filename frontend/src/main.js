/**
 * Net-Relay Dashboard
 * Frontend JavaScript for monitoring proxy connections
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
        this.init();
    }

    async init() {
        await this.checkHealth();
        await this.refresh();
        await this.loadHistory();
        
        // Start periodic refresh
        setInterval(() => this.refresh(), REFRESH_INTERVAL);
        setInterval(() => this.loadHistory(), REFRESH_INTERVAL * 5);
    }

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
