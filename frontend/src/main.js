/**
 * Net-Relay Dashboard
 * Frontend JavaScript for monitoring proxy connections and managing settings
 */

const API_BASE = '/api';
const REFRESH_INTERVAL = 2000; // 2 seconds

/**
 * Wrapper for fetch that handles authentication.
 */
async function apiFetch(url, options = {}) {
    const response = await fetch(url, {
        ...options,
        credentials: 'same-origin',
    });

    // Handle 401 Unauthorized - show login page
    if (response.status === 401) {
        showLoginPage();
        throw new Error('Authentication required');
    }

    return response;
}

/**
 * Show login page, hide dashboard.
 */
function showLoginPage() {
    document.getElementById('login-page').style.display = 'flex';
    document.getElementById('dashboard-container').style.display = 'none';
}

/**
 * Show dashboard, hide login page.
 */
function showDashboard() {
    document.getElementById('login-page').style.display = 'none';
    document.getElementById('dashboard-container').style.display = 'flex';
}

/**
 * Authentication Manager
 */
class AuthManager {
    constructor() {
        this.isAuthenticated = false;
        this.authEnabled = false;
        this.username = null;
    }

    async checkAuth() {
        try {
            const response = await fetch(`${API_BASE}/auth/check`, {
                credentials: 'same-origin',
            });
            const data = await response.json();

            if (data.success) {
                this.authEnabled = data.data.auth_enabled;
                this.isAuthenticated = data.data.authenticated;
                this.username = data.data.username;

                if (!this.authEnabled || this.isAuthenticated) {
                    showDashboard();
                    return true;
                } else {
                    showLoginPage();
                    return false;
                }
            }
        } catch (error) {
            console.error('Auth check failed:', error);
            // If auth check fails, assume no auth required
            showDashboard();
            return true;
        }
        return false;
    }

    async login(username, password) {
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'same-origin',
            body: JSON.stringify({ username, password }),
        });

        const data = await response.json();

        if (data.success && data.data.authenticated) {
            this.isAuthenticated = true;
            this.username = data.data.username;
            return { success: true };
        } else {
            return { success: false, error: data.message || 'Invalid credentials' };
        }
    }

    async logout() {
        try {
            await fetch(`${API_BASE}/auth/logout`, {
                method: 'POST',
                credentials: 'same-origin',
            });
        } catch (error) {
            console.error('Logout error:', error);
        }

        this.isAuthenticated = false;
        this.username = null;
        showLoginPage();
    }
}

// Global auth manager
const authManager = new AuthManager();

/**
 * Setup login form handlers
 */
function setupLoginForm() {
    const form = document.getElementById('login-form');
    const loginBtn = document.getElementById('login-btn');
    const errorDiv = document.getElementById('login-error');
    const btnText = loginBtn.querySelector('.btn-text');
    const btnLoading = loginBtn.querySelector('.btn-loading');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();

        const username = document.getElementById('login-username').value.trim();
        const password = document.getElementById('login-password').value;

        if (!username || !password) {
            showLoginError('Please enter username and password');
            return;
        }

        // Show loading state
        btnText.style.display = 'none';
        btnLoading.style.display = 'flex';
        loginBtn.disabled = true;
        errorDiv.style.display = 'none';

        try {
            const result = await authManager.login(username, password);

            if (result.success) {
                showDashboard();
                // Initialize dashboard after successful login
                if (!window.dashboard) {
                    window.dashboard = new Dashboard();
                } else {
                    window.dashboard.refresh();
                }
                // Clear form
                form.reset();
            } else {
                showLoginError(result.error);
            }
        } catch (error) {
            showLoginError('Network error. Please try again.');
            console.error('Login error:', error);
        } finally {
            // Reset button state
            btnText.style.display = 'inline';
            btnLoading.style.display = 'none';
            loginBtn.disabled = false;
        }
    });

    function showLoginError(message) {
        errorDiv.textContent = message;
        errorDiv.style.display = 'block';
        // Shake animation
        errorDiv.style.animation = 'none';
        errorDiv.offsetHeight;
        errorDiv.style.animation = 'shake 0.3s ease';
    }
}

/**
 * Setup logout button
 */
function setupLogoutButton() {
    const logoutBtn = document.getElementById('logout-btn');
    logoutBtn.addEventListener('click', () => {
        authManager.logout();
    });
}

class Dashboard {
    constructor() {
        this.elements = {
            status: document.getElementById('status'),
            authBadge: document.getElementById('auth-badge'),
            activeConnections: document.getElementById('active-connections'),
            totalConnections: document.getElementById('total-connections'),
            bytesSent: document.getElementById('bytes-sent'),
            bytesReceived: document.getElementById('bytes-received'),
            uptime: document.getElementById('uptime'),
            activeTbody: document.getElementById('active-tbody'),
            historyTbody: document.getElementById('history-tbody'),
            version: document.getElementById('version'),
            userStatsPanel: document.getElementById('user-stats-panel'),
            userStatsGrid: document.getElementById('user-stats-grid'),
            logoutBtn: document.getElementById('logout-btn'),
        };
        
        this.isConnected = false;
        this.accessControl = null;
        this.securityConfig = null;
        this.serverConfig = null;
        this.refreshInterval = null;
        this.historyInterval = null;
        this.init();
    }

    async init() {
        this.setupTabs();
        this.setupSettingsHandlers();
        this.setupUserHandlers();
        this.setupServerConfigHandlers();
        
        // Show/hide logout button based on auth status
        if (authManager.authEnabled) {
            this.elements.logoutBtn.style.display = 'inline-flex';
        }
        
        await this.checkHealth();
        await this.refresh();
        await this.loadHistory();
        await this.loadAccessControl();
        await this.loadSecurityConfig();
        await this.loadServerConfig();
        
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
        // IP Blacklist - button and Enter key
        const blacklistInput = document.getElementById('blacklist-ip-input');
        const addBlacklistBtn = document.getElementById('add-blacklist-btn');
        
        const handleAddBlacklist = () => {
            if (blacklistInput.value.trim()) {
                this.addIpBlacklist(blacklistInput.value.trim());
                blacklistInput.value = '';
                blacklistInput.focus();
            } else {
                this.shakeElement(blacklistInput);
            }
        };
        
        addBlacklistBtn?.addEventListener('click', handleAddBlacklist);
        blacklistInput?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') handleAddBlacklist();
        });

        // IP Whitelist - button and Enter key
        const whitelistInput = document.getElementById('whitelist-ip-input');
        const addWhitelistBtn = document.getElementById('add-whitelist-btn');
        
        const handleAddWhitelist = () => {
            if (whitelistInput.value.trim()) {
                this.addIpWhitelist(whitelistInput.value.trim());
                whitelistInput.value = '';
                whitelistInput.focus();
            } else {
                this.shakeElement(whitelistInput);
            }
        };
        
        addWhitelistBtn?.addEventListener('click', handleAddWhitelist);
        whitelistInput?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') handleAddWhitelist();
        });

        // Add Rule
        const addRuleBtn = document.getElementById('add-rule-btn');
        const domainInput = document.getElementById('rule-domain');
        
        const handleAddRule = () => {
            const name = document.getElementById('rule-name').value.trim();
            const domain = domainInput.value.trim();
            const path = document.getElementById('rule-path').value.trim() || null;
            const action = document.getElementById('rule-action').value;

            if (domain) {
                this.addRule({ name, domain, path, action, enabled: true });
                document.getElementById('rule-name').value = '';
                domainInput.value = '';
                document.getElementById('rule-path').value = '';
                document.getElementById('rule-name').focus();
            } else {
                this.shakeElement(domainInput);
                domainInput.focus();
            }
        };
        
        addRuleBtn?.addEventListener('click', handleAddRule);
        
        // Allow Enter on any rule form field
        ['rule-name', 'rule-domain', 'rule-path'].forEach(id => {
            document.getElementById(id)?.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') handleAddRule();
            });
        });
    }

    // Visual feedback for validation errors
    shakeElement(element) {
        element.style.animation = 'none';
        element.offsetHeight; // Trigger reflow
        element.style.animation = 'shake 0.3s ease';
        element.style.borderColor = 'var(--error)';
        setTimeout(() => {
            element.style.borderColor = '';
        }, 1000);
    }

    // ==================== Access Control API ====================

    async loadAccessControl() {
        try {
            const response = await apiFetch(`${API_BASE}/config/access-control`);
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
            modeDisplay.textContent = 'Blacklist Mode';
            modeDesc.textContent = 'All domains allowed except those blocked by rules below';
        } else {
            modeDisplay.textContent = 'Whitelist Mode';
            modeDesc.textContent = 'All domains blocked except those allowed by rules below';
        }

        // Render rules
        this.renderRules();
    }

    renderIpList(containerId, ips, type) {
        const container = document.getElementById(containerId);
        if (!container) return;

        if (!ips || ips.length === 0) {
            const emptyIcon = type === 'blacklist' ? '✓' : '🌐';
            const emptyText = type === 'blacklist' ? 'No blocked IPs' : 'All IPs allowed';
            container.innerHTML = `
                <div class="empty-state">
                    <span class="empty-icon">${emptyIcon}</span>
                    <span>${emptyText}</span>
                </div>
            `;
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
                const listType = btn.dataset.type;
                if (listType === 'blacklist') {
                    this.removeIpBlacklist(ip);
                } else {
                    this.removeIpWhitelist(ip);
                }
            });
        });
    }

    renderRules() {
        const tbody = document.getElementById('rules-tbody');
        const rulesCount = document.getElementById('rules-count');
        if (!tbody) return;

        const rules = this.accessControl.rules || [];
        
        // Update rules count
        if (rulesCount) {
            rulesCount.textContent = `${rules.length} rule${rules.length !== 1 ? 's' : ''}`;
        }
        
        if (rules.length === 0) {
            tbody.innerHTML = `
                <tr class="empty-row">
                    <td colspan="5">
                        <div class="table-empty-state">
                            <span class="empty-icon">📭</span>
                            <span>No rules configured</span>
                            <span class="empty-hint">Add a rule above to control domain access</span>
                        </div>
                    </td>
                </tr>
            `;
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
            const response = await apiFetch(`${API_BASE}/config/ip/blacklist`, {
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
            const response = await apiFetch(`${API_BASE}/config/ip/blacklist`, {
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
            const response = await apiFetch(`${API_BASE}/config/ip/whitelist`, {
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
            const response = await apiFetch(`${API_BASE}/config/ip/whitelist`, {
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
            const response = await apiFetch(`${API_BASE}/config/rules`, {
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
            const response = await apiFetch(`${API_BASE}/config/rules`, {
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

    // ==================== Security & User Management ====================

    setupUserHandlers() {
        // Auth toggle
        const authToggle = document.getElementById('auth-enabled-toggle');
        authToggle?.addEventListener('change', () => {
            this.updateAuthEnabled(authToggle.checked);
        });

        // Add user
        const addUserBtn = document.getElementById('add-user-btn');
        const usernameInput = document.getElementById('user-username');
        const passwordInput = document.getElementById('user-password');
        
        const handleAddUser = () => {
            const username = usernameInput?.value.trim();
            const password = passwordInput?.value;
            const description = document.getElementById('user-description')?.value.trim() || null;
            
            if (username && password) {
                this.addUser({ username, password, description });
                usernameInput.value = '';
                passwordInput.value = '';
                document.getElementById('user-description').value = '';
                usernameInput.focus();
            } else {
                if (!username) this.shakeElement(usernameInput);
                if (!password) this.shakeElement(passwordInput);
            }
        };
        
        addUserBtn?.addEventListener('click', handleAddUser);
        
        // Allow Enter on user form fields
        ['user-username', 'user-password', 'user-description'].forEach(id => {
            document.getElementById(id)?.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') handleAddUser();
            });
        });
    }

    async loadSecurityConfig() {
        try {
            const response = await apiFetch(`${API_BASE}/config/security`);
            const data = await response.json();
            
            if (data.success) {
                this.securityConfig = data.data;
                this.renderSecurityConfig();
            }
        } catch (error) {
            console.error('Failed to load security config:', error);
        }
    }

    renderSecurityConfig() {
        if (!this.securityConfig) return;

        // Update auth badge in header
        const authBadge = this.elements.authBadge;
        if (authBadge) {
            if (this.securityConfig.auth_enabled) {
                authBadge.classList.remove('auth-off');
                authBadge.classList.add('auth-on');
                authBadge.querySelector('.auth-icon').textContent = '🔒';
                authBadge.querySelector('.auth-text').textContent = 'Auth On';
            } else {
                authBadge.classList.remove('auth-on');
                authBadge.classList.add('auth-off');
                authBadge.querySelector('.auth-icon').textContent = '🔓';
                authBadge.querySelector('.auth-text').textContent = 'Auth Off';
            }
        }

        // Update auth toggle
        const authToggle = document.getElementById('auth-enabled-toggle');
        const authToggleLabel = document.getElementById('auth-toggle-label');
        if (authToggle) {
            authToggle.checked = this.securityConfig.auth_enabled;
        }
        if (authToggleLabel) {
            authToggleLabel.textContent = this.securityConfig.auth_enabled ? 'Enabled' : 'Disabled';
        }

        // Render users table
        this.renderUsersTable();
    }

    renderUsersTable() {
        const tbody = document.getElementById('users-tbody');
        const usersCount = document.getElementById('users-count');
        if (!tbody || !this.securityConfig) return;

        const users = this.securityConfig.users || [];
        
        // Update count
        if (usersCount) {
            usersCount.textContent = `${users.length} user${users.length !== 1 ? 's' : ''}`;
        }
        
        if (users.length === 0) {
            tbody.innerHTML = `
                <tr class="empty-row">
                    <td colspan="4">
                        <div class="table-empty-state">
                            <span class="empty-icon">👤</span>
                            <span>No users configured</span>
                            <span class="empty-hint">Add a user above to enable authentication</span>
                        </div>
                    </td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = users.map(user => `
            <tr>
                <td><strong>${this.escapeHtml(user.username)}</strong></td>
                <td>${this.escapeHtml(user.description || '-')}</td>
                <td>
                    <span class="user-status-badge ${user.enabled ? 'enabled' : 'disabled'}">
                        ${user.enabled ? 'Active' : 'Disabled'}
                    </span>
                </td>
                <td>
                    <button class="btn btn-sm btn-danger remove-user" data-username="${this.escapeHtml(user.username)}">
                        Remove
                    </button>
                </td>
            </tr>
        `).join('');

        // Add remove handlers
        tbody.querySelectorAll('.remove-user').forEach(btn => {
            btn.addEventListener('click', () => {
                this.removeUser(btn.dataset.username);
            });
        });
    }

    async updateAuthEnabled(enabled) {
        try {
            const response = await apiFetch(`${API_BASE}/config/security`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ auth_enabled: enabled })
            });
            const data = await response.json();
            if (data.success) {
                this.securityConfig = data.data;
                this.renderSecurityConfig();
            }
        } catch (error) {
            console.error('Failed to update auth setting:', error);
        }
    }

    async addUser(user) {
        try {
            const response = await apiFetch(`${API_BASE}/config/users`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(user)
            });
            const data = await response.json();
            if (data.success) {
                this.securityConfig = data.data;
                this.renderSecurityConfig();
            } else if (data.message) {
                alert(data.message);
            }
        } catch (error) {
            console.error('Failed to add user:', error);
        }
    }

    async removeUser(username) {
        if (!confirm(`Remove user "${username}"?`)) return;
        
        try {
            const response = await apiFetch(`${API_BASE}/config/users`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username })
            });
            const data = await response.json();
            if (data.success) {
                this.securityConfig = data.data;
                this.renderSecurityConfig();
            }
        } catch (error) {
            console.error('Failed to remove user:', error);
        }
    }

    // ==================== Server Configuration API ====================

    setupServerConfigHandlers() {
        const saveBtn = document.getElementById('save-server-config-btn');
        if (saveBtn) {
            saveBtn.addEventListener('click', () => this.saveServerConfig());
        }
    }

    async loadServerConfig() {
        try {
            const response = await apiFetch(`${API_BASE}/config/server`);
            const data = await response.json();
            
            if (data.success) {
                this.serverConfig = data.data;
                this.renderServerConfig();
            }
        } catch (error) {
            console.error('Failed to load server config:', error);
        }
    }

    renderServerConfig() {
        if (!this.serverConfig) return;

        const hostInput = document.getElementById('server-host');
        const socksPortInput = document.getElementById('server-socks-port');
        const httpPortInput = document.getElementById('server-http-port');
        const apiPortInput = document.getElementById('server-api-port');
        const restartWarning = document.getElementById('restart-warning');

        if (hostInput) hostInput.value = this.serverConfig.host;
        if (socksPortInput) socksPortInput.value = this.serverConfig.socks_port;
        if (httpPortInput) httpPortInput.value = this.serverConfig.http_port;
        if (apiPortInput) apiPortInput.value = this.serverConfig.api_port;
        if (restartWarning) restartWarning.style.display = 'none';
    }

    async saveServerConfig() {
        const hostInput = document.getElementById('server-host');
        const socksPortInput = document.getElementById('server-socks-port');
        const httpPortInput = document.getElementById('server-http-port');
        const apiPortInput = document.getElementById('server-api-port');
        const statusEl = document.getElementById('server-save-status');
        const restartWarning = document.getElementById('restart-warning');
        const saveBtn = document.getElementById('save-server-config-btn');

        // Validate
        const host = hostInput ? hostInput.value.trim() : '0.0.0.0';
        const socksPort = socksPortInput ? parseInt(socksPortInput.value) : 1080;
        const httpPort = httpPortInput ? parseInt(httpPortInput.value) : 8080;
        const apiPort = apiPortInput ? parseInt(apiPortInput.value) : 3000;

        if (!host) {
            this.shakeElement(hostInput);
            return;
        }

        const validatePort = (port, input) => {
            if (isNaN(port) || port < 1 || port > 65535) {
                this.shakeElement(input);
                return false;
            }
            return true;
        };

        if (!validatePort(socksPort, socksPortInput) ||
            !validatePort(httpPort, httpPortInput) ||
            !validatePort(apiPort, apiPortInput)) {
            return;
        }

        // Show saving state
        if (saveBtn) saveBtn.disabled = true;
        if (statusEl) {
            statusEl.textContent = 'Saving...';
            statusEl.className = 'save-status';
        }

        try {
            const response = await apiFetch(`${API_BASE}/config/server`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    host,
                    socks_port: socksPort,
                    http_port: httpPort,
                    api_port: apiPort
                })
            });
            const data = await response.json();

            if (data.success) {
                this.serverConfig = data.data;
                if (statusEl) {
                    statusEl.textContent = '✓ Saved';
                    statusEl.className = 'save-status success';
                }
                if (restartWarning && data.data.requires_restart) {
                    restartWarning.style.display = 'block';
                }
                setTimeout(() => {
                    if (statusEl) statusEl.textContent = '';
                }, 3000);
            } else {
                if (statusEl) {
                    statusEl.textContent = data.message || 'Save failed';
                    statusEl.className = 'save-status error';
                }
            }
        } catch (error) {
            console.error('Failed to save server config:', error);
            if (statusEl) {
                statusEl.textContent = 'Save failed';
                statusEl.className = 'save-status error';
            }
        } finally {
            if (saveBtn) saveBtn.disabled = false;
        }
    }

    // ==================== Dashboard Stats ====================

    async checkHealth() {
        try {
            const response = await apiFetch(`${API_BASE}/health`);
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
            const response = await apiFetch(`${API_BASE}/stats`);
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

        // Update per-user stats
        if (stats.users && stats.users.length > 0) {
            this.renderUserStats(stats.users);
        } else {
            this.elements.userStatsPanel.style.display = 'none';
        }
    }

    renderUserStats(users) {
        const panel = this.elements.userStatsPanel;
        const grid = this.elements.userStatsGrid;
        
        if (!panel || !grid) return;
        
        panel.style.display = 'block';
        
        grid.innerHTML = users.map(user => `
            <div class="user-stat-card">
                <div class="user-stat-header">
                    <div class="user-stat-name">
                        <div class="user-stat-avatar">${user.username.charAt(0).toUpperCase()}</div>
                        <h4>${this.escapeHtml(user.username)}</h4>
                    </div>
                    <span class="user-stat-active ${user.active_connections > 0 ? 'has-active' : 'no-active'}">
                        ${user.active_connections} active
                    </span>
                </div>
                <div class="user-stat-details">
                    <div class="user-stat-item">
                        <span class="user-stat-value">${user.total_connections}</span>
                        <span class="user-stat-label">Connections</span>
                    </div>
                    <div class="user-stat-item">
                        <span class="user-stat-value">${this.formatBytes(user.total_bytes_sent)}</span>
                        <span class="user-stat-label">Sent</span>
                    </div>
                    <div class="user-stat-item">
                        <span class="user-stat-value">${this.formatBytes(user.total_bytes_received)}</span>
                        <span class="user-stat-label">Received</span>
                    </div>
                </div>
            </div>
        `).join('');
    }

    updateActiveConnections(connections) {
        const tbody = this.elements.activeTbody;
        
        if (connections.length === 0) {
            tbody.innerHTML = '<tr class="empty-row"><td colspan="7">No active connections</td></tr>';
            return;
        }

        tbody.innerHTML = connections.map(conn => `
            <tr>
                <td><span class="protocol-badge ${conn.protocol}">${conn.protocol}</span></td>
                <td>${this.escapeHtml(conn.client_addr)}</td>
                <td>${this.escapeHtml(conn.target_addr)}:${conn.target_port}</td>
                <td class="user-cell ${conn.username ? '' : 'anonymous'}">${conn.username ? this.escapeHtml(conn.username) : '-'}</td>
                <td>${this.formatDuration(this.calculateDuration(conn.connected_at))}</td>
                <td>${this.formatBytes(conn.bytes_sent)}</td>
                <td>${this.formatBytes(conn.bytes_received)}</td>
            </tr>
        `).join('');
    }

    async loadHistory() {
        try {
            const response = await apiFetch(`${API_BASE}/history?limit=50`);
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
            tbody.innerHTML = '<tr class="empty-row"><td colspan="8">No connection history</td></tr>';
            return;
        }

        tbody.innerHTML = history.map(item => {
            const conn = item;
            return `
                <tr>
                    <td><span class="protocol-badge ${conn.protocol}">${conn.protocol}</span></td>
                    <td>${this.escapeHtml(conn.client_addr)}</td>
                    <td>${this.escapeHtml(conn.target_addr)}:${conn.target_port}</td>
                    <td class="user-cell ${conn.username ? '' : 'anonymous'}">${conn.username ? this.escapeHtml(conn.username) : '-'}</td>
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

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', async () => {
    // Setup login form handlers first
    setupLoginForm();
    setupLogoutButton();

    // Check auth status and show appropriate view
    const authenticated = await authManager.checkAuth();

    // Initialize dashboard if authenticated or no auth required
    if (authenticated) {
        window.dashboard = new Dashboard();
    }
});
