/**
 * Server List Component (Feature 031)
 *
 * Displays a list of servers with sorting and selection.
 */

/**
 * Server list component
 */
export class ServerList {
    /**
     * @param {HTMLElement} container - Container element
     * @param {object} options - List options
     * @param {Function} options.onSelect - Selection callback (server)
     * @param {Function} options.onConnect - Connect callback (server)
     * @param {Function} options.onToggleFavorite - Favorite toggle callback (address)
     */
    constructor(container, options = {}) {
        this.container = container;
        this.options = options;

        /** @type {Array} Server list */
        this.servers = [];

        /** @type {string|null} Selected server address */
        this.selected = null;

        /** @type {string} Sort field */
        this.sortField = 'name';

        /** @type {boolean} Sort ascending */
        this.sortAsc = true;
    }

    /**
     * Render the server list
     * @param {Array} servers - Server data
     */
    render(servers) {
        this.servers = servers;

        if (servers.length === 0) {
            this.container.innerHTML = `
                <div class="empty-state">
                    No servers found
                </div>
            `;
            return;
        }

        // Sort servers
        const sorted = this.sortServers([...servers]);

        this.container.innerHTML = `
            <div class="server-list">
                <div class="server-list-header">
                    <span class="sortable" data-sort="name">Name</span>
                    <span class="sortable" data-sort="region">Region</span>
                    <span>Tags</span>
                    <span class="sortable" data-sort="players">Players</span>
                    <span></span>
                </div>
                ${sorted.map((s) => this.renderRow(s)).join('')}
            </div>
        `;

        this.attachListeners();
    }

    /**
     * Render a server row
     * @param {object} server - Server data
     * @returns {string} HTML string
     */
    renderRow(server) {
        const isSelected = this.selected === server.address;
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

    /**
     * Sort servers
     * @param {Array} servers - Server array
     * @returns {Array} Sorted array
     */
    sortServers(servers) {
        return servers.sort((a, b) => {
            // Favorites always at top
            if (a.is_favorite && !b.is_favorite) return -1;
            if (!a.is_favorite && b.is_favorite) return 1;

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
            return this.sortAsc ? cmp : -cmp;
        });
    }

    /**
     * Attach event listeners
     */
    attachListeners() {
        // Row click - select
        this.container.querySelectorAll('.server-row').forEach((row) => {
            row.addEventListener('click', (e) => {
                if (e.target.closest('.server-favorite')) return;

                const address = row.dataset.address;
                this.select(address);
            });

            // Double-click - connect
            row.addEventListener('dblclick', (e) => {
                if (e.target.closest('.server-favorite')) return;

                const address = row.dataset.address;
                const server = this.servers.find((s) => s.address === address);
                if (server && this.options.onConnect) {
                    this.options.onConnect(server);
                }
            });
        });

        // Favorite toggle
        this.container.querySelectorAll('.server-favorite').forEach((btn) => {
            btn.addEventListener('click', (e) => {
                e.stopPropagation();
                const address = btn.dataset.address;
                if (this.options.onToggleFavorite) {
                    this.options.onToggleFavorite(address);
                }
            });
        });

        // Sort header clicks
        this.container.querySelectorAll('.sortable').forEach((header) => {
            header.addEventListener('click', () => {
                const field = header.dataset.sort;
                if (this.sortField === field) {
                    this.sortAsc = !this.sortAsc;
                } else {
                    this.sortField = field;
                    this.sortAsc = true;
                }
                this.render(this.servers);
            });
        });
    }

    /**
     * Select a server
     * @param {string} address - Server address
     */
    select(address) {
        this.selected = address;

        // Update UI
        this.container.querySelectorAll('.server-row').forEach((row) => {
            row.classList.toggle('selected', row.dataset.address === address);
        });

        // Callback
        const server = this.servers.find((s) => s.address === address);
        if (server && this.options.onSelect) {
            this.options.onSelect(server);
        }
    }

    /**
     * Get selected server
     * @returns {object|null}
     */
    getSelected() {
        return this.servers.find((s) => s.address === this.selected) || null;
    }

    /**
     * Clear selection
     */
    clearSelection() {
        this.selected = null;
        this.container.querySelectorAll('.server-row.selected').forEach((row) => {
            row.classList.remove('selected');
        });
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
