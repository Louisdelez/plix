/**
 * Server Browser Page (Feature 031)
 *
 * Browse, filter, and connect to game servers.
 */

// Search debounce delay in ms
const SEARCH_DEBOUNCE = 300;

// Button debounce delay in ms
const BUTTON_DEBOUNCE = 500;

export class ServersPage {
    /**
     * @param {object} context - Application context
     */
    constructor(context) {
        this.navigate = context.navigate;
        this.bridge = context.bridge;
        this.state = context.state;

        /** @type {Array} Server list */
        this.servers = [];

        /** @type {Array} Filtered server list */
        this.filteredServers = [];

        /** @type {string} Search query */
        this.searchQuery = '';

        /** @type {string} Region filter */
        this.regionFilter = '';

        /** @type {boolean} Show favorites only */
        this.favoritesOnly = false;

        /** @type {boolean} Show incompatible servers */
        this.showIncompatible = true;

        /** @type {string|null} Selected server address */
        this.selectedServer = null;

        /** @type {string} Sort field */
        this.sortField = 'name';

        /** @type {boolean} Sort ascending */
        this.sortAsc = true;

        /** @type {boolean} Loading state */
        this.loading = false;

        /** @type {string|null} Error message */
        this.error = null;

        /** @type {object|null} Connection modal state */
        this.connectModal = null;

        /** @type {number|null} Search debounce timer */
        this._searchTimer = null;

        /** @type {number} Last button click time */
        this._lastClick = 0;
    }

    render() {
        return `
            <div class="page">
                <div class="page-header">
                    <h1 class="page-title">Server Browser</h1>
                </div>

                <div class="filter-bar">
                    <input type="text" class="form-input search-input" id="search"
                        placeholder="Search servers..." maxlength="32">
                    <select class="filter-select" id="region-filter">
                        <option value="">All Regions</option>
                    </select>
                    <label class="checkbox-container">
                        <input type="checkbox" class="checkbox-input" id="favorites-only">
                        <span class="checkbox-label">Favorites</span>
                    </label>
                    <button class="btn" id="btn-refresh">Refresh</button>
                </div>

                <div id="server-list-container">
                    <div class="loading">
                        <div class="loading-spinner"></div>
                        <div class="loading-text">Loading servers...</div>
                    </div>
                </div>

                <div class="action-bar">
                    <button class="btn" id="btn-back">Back</button>
                    <button class="btn btn-primary" id="btn-connect" disabled>Connect</button>
                </div>

                <div id="modal-container"></div>
            </div>
        `;
    }

    async init() {
        // Set up back button
        document.getElementById('btn-back')?.addEventListener('click', () => {
            this.navigate('/');
        });

        // Set up connect button
        document.getElementById('btn-connect')?.addEventListener('click', () => {
            this.connectToSelected();
        });

        // Set up refresh button
        document.getElementById('btn-refresh')?.addEventListener('click', () => {
            if (this.debounceClick()) {
                this.refresh();
            }
        });

        // Set up search input with debounce
        document.getElementById('search')?.addEventListener('input', (e) => {
            clearTimeout(this._searchTimer);
            this._searchTimer = setTimeout(() => {
                this.searchQuery = e.target.value.trim().toLowerCase();
                this.applyFilters();
            }, SEARCH_DEBOUNCE);
        });

        // Set up region filter
        document.getElementById('region-filter')?.addEventListener('change', (e) => {
            this.regionFilter = e.target.value;
            this.applyFilters();
        });

        // Set up favorites filter
        document.getElementById('favorites-only')?.addEventListener('change', (e) => {
            this.favoritesOnly = e.target.checked;
            this.applyFilters();
        });

        // Listen for connection status updates
        window.addEventListener('plix:connection-status', (e) => {
            this.handleConnectionStatus(e.detail);
        });

        // Listen for favorites updates
        window.addEventListener('plix:favorites-updated', (e) => {
            this.handleFavoritesUpdated(e.detail);
        });

        // Initial load
        await this.refresh();
    }

    debounceClick() {
        const now = Date.now();
        if (now - this._lastClick < BUTTON_DEBOUNCE) {
            return false;
        }
        this._lastClick = now;
        return true;
    }

    async refresh() {
        const refreshBtn = document.getElementById('btn-refresh');
        if (refreshBtn) {
            refreshBtn.disabled = true;
            refreshBtn.classList.add('btn-loading');
        }

        this.loading = true;
        this.error = null;

        try {
            const payload = await this.bridge.send('FetchServers', {});
            this.servers = payload.servers || [];
            this.populateRegionFilter();
            this.applyFilters();
        } catch (e) {
            this.error = e.message || 'Failed to fetch servers';
            this.renderServerList();
        } finally {
            this.loading = false;
            if (refreshBtn) {
                refreshBtn.disabled = false;
                refreshBtn.classList.remove('btn-loading');
            }
        }
    }

    populateRegionFilter() {
        const regions = [...new Set(this.servers.map((s) => s.region).filter(Boolean))];
        const select = document.getElementById('region-filter');
        if (!select) return;

        // Keep "All Regions" option
        select.innerHTML =
            '<option value="">All Regions</option>' +
            regions.map((r) => `<option value="${escapeHtml(r)}">${escapeHtml(r)}</option>`).join('');
    }

    applyFilters() {
        let filtered = [...this.servers];

        // Search filter
        if (this.searchQuery) {
            filtered = filtered.filter(
                (s) =>
                    s.name.toLowerCase().includes(this.searchQuery) ||
                    s.region?.toLowerCase().includes(this.searchQuery) ||
                    s.tags?.some((t) => t.toLowerCase().includes(this.searchQuery))
            );
        }

        // Region filter
        if (this.regionFilter) {
            filtered = filtered.filter((s) => s.region === this.regionFilter);
        }

        // Favorites filter
        if (this.favoritesOnly) {
            filtered = filtered.filter((s) => s.is_favorite);
        }

        // Hide incompatible
        if (!this.showIncompatible) {
            filtered = filtered.filter((s) => s.is_compatible);
        }

        // Sort
        filtered.sort((a, b) => {
            let cmp = 0;
            switch (this.sortField) {
                case 'name':
                    cmp = a.name.localeCompare(b.name);
                    break;
                case 'players':
                    cmp = a.players - b.players;
                    break;
                case 'region':
                    cmp = (a.region || '').localeCompare(b.region || '');
                    break;
            }
            // Favorites always at top
            if (a.is_favorite && !b.is_favorite) return -1;
            if (!a.is_favorite && b.is_favorite) return 1;
            return this.sortAsc ? cmp : -cmp;
        });

        this.filteredServers = filtered;
        this.renderServerList();
    }

    renderServerList() {
        const container = document.getElementById('server-list-container');
        if (!container) return;

        if (this.loading) {
            container.innerHTML = `
                <div class="loading">
                    <div class="loading-spinner"></div>
                    <div class="loading-text">Loading servers...</div>
                </div>
            `;
            return;
        }

        if (this.error) {
            container.innerHTML = `
                <div class="error-message">${escapeHtml(this.error)}</div>
            `;
            return;
        }

        if (this.filteredServers.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    ${this.servers.length === 0 ? 'No servers found. Click Refresh to fetch the server list.' : 'No servers match your filters.'}
                </div>
            `;
            return;
        }

        container.innerHTML = `
            <div class="server-list">
                <div class="server-list-header">
                    <span>Name</span>
                    <span>Region</span>
                    <span>Tags</span>
                    <span>Players</span>
                    <span></span>
                </div>
                ${this.filteredServers.map((s) => this.renderServerRow(s)).join('')}
            </div>
        `;

        // Attach row click handlers
        container.querySelectorAll('.server-row').forEach((row) => {
            row.addEventListener('click', (e) => {
                // Ignore clicks on favorite button
                if (e.target.closest('.server-favorite')) return;

                const address = row.dataset.address;
                this.selectServer(address);
            });

            row.addEventListener('dblclick', (e) => {
                if (e.target.closest('.server-favorite')) return;
                const address = row.dataset.address;
                this.selectServer(address);
                this.connectToSelected();
            });
        });

        // Attach favorite button handlers
        container.querySelectorAll('.server-favorite').forEach((btn) => {
            btn.addEventListener('click', (e) => {
                e.stopPropagation();
                const address = btn.dataset.address;
                this.toggleFavorite(address);
            });
        });
    }

    renderServerRow(server) {
        const isSelected = this.selectedServer === server.address;
        const compatClass = server.is_compatible ? '' : 'incompatible';

        return `
            <div class="server-row ${isSelected ? 'selected' : ''} ${compatClass}"
                data-address="${escapeHtml(server.address)}">
                <div class="server-name">
                    ${escapeHtml(server.name)}
                    ${!server.is_compatible ? '<span class="badge badge-warning" title="Version mismatch">!</span>' : ''}
                </div>
                <div class="server-region">${escapeHtml(server.region || '-')}</div>
                <div class="server-tags">
                    ${(server.tags || []).slice(0, 2).map((t) => `<span class="tag">${escapeHtml(t)}</span>`).join(' ')}
                </div>
                <div class="server-players">${server.players}/${server.max_players}</div>
                <div class="server-favorite ${server.is_favorite ? 'active' : ''}"
                    data-address="${escapeHtml(server.address)}"
                    title="${server.is_favorite ? 'Remove from favorites' : 'Add to favorites'}">
                    ★
                </div>
            </div>
        `;
    }

    selectServer(address) {
        this.selectedServer = address;

        // Update UI
        document.querySelectorAll('.server-row').forEach((row) => {
            row.classList.toggle('selected', row.dataset.address === address);
        });

        // Enable/disable connect button
        const server = this.servers.find((s) => s.address === address);
        const connectBtn = document.getElementById('btn-connect');
        if (connectBtn) {
            connectBtn.disabled = !server || !server.is_compatible;
        }
    }

    async toggleFavorite(address) {
        try {
            const result = await this.bridge.send('ToggleFavorite', { address });

            // Update local state
            const server = this.servers.find((s) => s.address === address);
            if (server) {
                server.is_favorite = result.is_favorite;
            }

            // Re-render
            this.applyFilters();
        } catch (e) {
            console.error('[Plix] Toggle favorite failed:', e);
        }
    }

    handleFavoritesUpdated(payload) {
        // Sync favorites from external update
        const favoriteAddresses = new Set(payload.favorites || []);
        this.servers.forEach((s) => {
            s.is_favorite = favoriteAddresses.has(s.address);
        });
        this.applyFilters();
    }

    async connectToSelected() {
        if (!this.selectedServer) return;

        const server = this.servers.find((s) => s.address === this.selectedServer);
        if (!server) return;

        if (!server.is_compatible) {
            this.showModal(
                'Version Mismatch',
                'This server is running an incompatible version. Cannot connect.',
                [{ label: 'OK', primary: true, action: () => this.hideModal() }]
            );
            return;
        }

        // Show connecting modal
        this.connectModal = {
            address: server.address,
            name: server.name,
            state: 'connecting',
            message: 'Connecting to server...',
        };
        this.renderConnectModal();

        try {
            await this.bridge.send('Connect', { address: server.address });
            // Connection status will be updated via push events
        } catch (e) {
            this.connectModal.state = 'failed';
            this.connectModal.message = e.message || 'Connection failed';
            this.renderConnectModal();
        }
    }

    handleConnectionStatus(payload) {
        if (!this.connectModal) return;

        this.connectModal.state = payload.state;
        this.connectModal.message = payload.message;

        if (payload.state === 'connected') {
            // Connection successful - game will transition
            setTimeout(() => this.hideModal(), 1000);
        } else if (payload.state === 'failed' || payload.state === 'disconnected') {
            this.renderConnectModal();
        } else {
            this.renderConnectModal();
        }
    }

    renderConnectModal() {
        if (!this.connectModal) return;

        const { name, state, message } = this.connectModal;

        let content = '';
        let actions = [];

        switch (state) {
            case 'connecting':
                content = `
                    <div class="loading">
                        <div class="loading-spinner"></div>
                        <div class="loading-text">${escapeHtml(message)}</div>
                    </div>
                `;
                actions = [{ label: 'Cancel', action: () => this.hideModal() }];
                break;
            case 'connected':
                content = `
                    <div class="success-message">${escapeHtml(message)}</div>
                `;
                break;
            case 'failed':
            case 'disconnected':
                content = `
                    <div class="error-message">${escapeHtml(message)}</div>
                `;
                actions = [
                    { label: 'Retry', primary: true, action: () => this.connectToSelected() },
                    { label: 'Close', action: () => this.hideModal() },
                ];
                break;
        }

        this.showModal(`Connecting to ${name}`, content, actions, true);
    }

    showModal(title, content, actions = [], isRaw = false) {
        const container = document.getElementById('modal-container');
        if (!container) return;

        container.innerHTML = `
            <div class="modal-overlay">
                <div class="modal">
                    <div class="modal-title">${escapeHtml(title)}</div>
                    <div class="modal-content">${isRaw ? content : escapeHtml(content)}</div>
                    <div class="modal-actions">
                        ${actions.map((a, i) => `
                            <button class="btn ${a.primary ? 'btn-primary' : ''}" data-action="${i}">
                                ${escapeHtml(a.label)}
                            </button>
                        `).join('')}
                    </div>
                </div>
            </div>
        `;

        // Attach action handlers
        container.querySelectorAll('[data-action]').forEach((btn) => {
            const idx = parseInt(btn.dataset.action, 10);
            btn.addEventListener('click', () => {
                if (actions[idx]?.action) {
                    actions[idx].action();
                }
            });
        });
    }

    hideModal() {
        this.connectModal = null;
        const container = document.getElementById('modal-container');
        if (container) {
            container.innerHTML = '';
        }
    }
}

/**
 * Escape HTML special characters
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
    if (str === null || str === undefined) return '';
    const div = document.createElement('div');
    div.textContent = String(str);
    return div.innerHTML;
}
