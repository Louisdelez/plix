/**
 * Main Menu Page (Feature 031)
 *
 * Entry point to the game - Play, Servers, Settings, Quit
 */

export class MainPage {
    /**
     * @param {object} context - Application context
     * @param {Function} context.navigate - Navigation function
     * @param {object} context.bridge - Bridge interface
     * @param {object} context.state - Application state
     */
    constructor(context) {
        this.navigate = context.navigate;
        this.bridge = context.bridge;
        this.state = context.state;
    }

    /**
     * Render the page HTML
     * @returns {string}
     */
    render() {
        const playerName = this.state.playerName || 'Player';

        return `
            <div class="page">
                <div class="page-header">
                    <h1 class="page-title">PLIX</h1>
                    <p class="page-subtitle">Multiplayer Voxel Arena</p>
                </div>

                <div class="menu-buttons">
                    <button class="btn btn-large btn-primary" id="btn-play" disabled title="Coming soon">
                        Play
                    </button>
                    <button class="btn btn-large" id="btn-servers">
                        Servers
                    </button>
                    <button class="btn btn-large" id="btn-settings">
                        Settings
                    </button>
                    <button class="btn btn-large" id="btn-quit">
                        Quit
                    </button>
                </div>

                <p class="player-name text-center">
                    Playing as: <strong>${escapeHtml(playerName)}</strong>
                </p>
            </div>
        `;
    }

    /**
     * Initialize event listeners
     */
    init() {
        // Servers button
        document.getElementById('btn-servers')?.addEventListener('click', () => {
            this.navigate('/servers');
        });

        // Settings button
        document.getElementById('btn-settings')?.addEventListener('click', () => {
            this.navigate('/settings');
        });

        // Quit button
        document.getElementById('btn-quit')?.addEventListener('click', async () => {
            const btn = document.getElementById('btn-quit');
            if (btn) {
                btn.disabled = true;
                btn.classList.add('btn-loading');
            }

            try {
                await this.bridge.send('Quit', {});
                // Game should exit - but in dev mode, show message
                console.log('[Plix] Quit acknowledged');
            } catch (e) {
                console.error('[Plix] Quit failed:', e);
                if (btn) {
                    btn.disabled = false;
                    btn.classList.remove('btn-loading');
                }
            }
        });
    }
}

/**
 * Escape HTML special characters
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}
