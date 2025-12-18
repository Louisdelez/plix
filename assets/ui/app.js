/**
 * Plix CEF UI - Main Application (Feature 031)
 *
 * Hash-based SPA router with JS↔Rust bridge integration.
 */

// Import page modules
import { MainPage } from './pages/main.js';
import { SettingsPage } from './pages/settings.js';
import { ServersPage } from './pages/servers.js';

// Bridge version for handshake
const BRIDGE_VERSION = '1.0';

// Application state
const state = {
    /** @type {string} Current route path */
    currentRoute: '/',

    /** @type {string|null} Player display name from handshake */
    playerName: null,

    /** @type {boolean} Bridge handshake completed */
    bridgeReady: false,

    /** @type {Map<string, Function>} Pending request callbacks */
    pendingRequests: new Map(),

    /** @type {number} Request ID counter */
    requestId: 0,
};

// Route definitions
const routes = {
    '/': MainPage,
    '/settings': SettingsPage,
    '/servers': ServersPage,
};

/**
 * Generate unique request ID
 * @returns {string}
 */
function generateRequestId() {
    return `req-${++state.requestId}`;
}

/**
 * Bridge interface for communication with game engine
 */
const bridge = {
    /**
     * Send message to game engine
     * @param {string} type - Message type
     * @param {object} payload - Message payload
     * @returns {Promise<object>} Response payload
     */
    send(type, payload = {}) {
        return new Promise((resolve, reject) => {
            const id = generateRequestId();
            const message = { id, type, payload };

            // Store callback for response
            state.pendingRequests.set(id, { resolve, reject });

            // Set timeout for request
            setTimeout(() => {
                if (state.pendingRequests.has(id)) {
                    state.pendingRequests.delete(id);
                    reject(new Error('Request timeout'));
                }
            }, 10000);

            // Send via native bridge if available
            if (window.plix && typeof window.plix.send === 'function') {
                try {
                    window.plix.send(JSON.stringify(message));
                } catch (e) {
                    state.pendingRequests.delete(id);
                    reject(e);
                }
            } else {
                // Development fallback - simulate responses
                console.log('[Plix Bridge] Send:', type, payload);
                simulateResponse(id, type, payload);
            }
        });
    },

    /**
     * Handle incoming message from game engine
     * @param {string} jsonStr - JSON message string
     */
    handleMessage(jsonStr) {
        try {
            const msg = JSON.parse(jsonStr);

            // Check if this is a response to a pending request
            if (msg.id && state.pendingRequests.has(msg.id)) {
                const { resolve, reject } = state.pendingRequests.get(msg.id);
                state.pendingRequests.delete(msg.id);

                if (msg.ok) {
                    resolve(msg.payload);
                } else {
                    reject(new Error(msg.error?.message || 'Unknown error'));
                }
                return;
            }

            // Handle push events (no id)
            if (!msg.id) {
                handlePushEvent(msg.type, msg.payload);
            }
        } catch (e) {
            console.error('[Plix Bridge] Failed to parse message:', e);
        }
    },
};

/**
 * Handle push events from game engine
 * @param {string} type - Event type
 * @param {object} payload - Event payload
 */
function handlePushEvent(type, payload) {
    console.log('[Plix Bridge] Push event:', type, payload);

    switch (type) {
        case 'ConnectionStatus':
            // Dispatch custom event for server browser to handle
            window.dispatchEvent(
                new CustomEvent('plix:connection-status', { detail: payload })
            );
            break;
        case 'FavoritesUpdated':
            window.dispatchEvent(
                new CustomEvent('plix:favorites-updated', { detail: payload })
            );
            break;
        default:
            console.warn('[Plix Bridge] Unknown push event:', type);
    }
}

/**
 * Development mode - simulate responses for testing without game engine
 * @param {string} id - Request ID
 * @param {string} type - Message type
 * @param {object} payload - Request payload
 */
function simulateResponse(id, type, payload) {
    setTimeout(() => {
        let response;

        switch (type) {
            case 'Handshake':
                response = {
                    id,
                    type,
                    ok: true,
                    payload: {
                        supported_version: '1.0',
                        display_name: 'DevPlayer',
                    },
                };
                break;
            case 'GetConfig':
                response = {
                    id,
                    type,
                    ok: true,
                    payload: {
                        sensitivity: 0.003,
                        fov_degrees: 70,
                        fullscreen: false,
                        audio_muted: false,
                        keybinds: {
                            forward: 'W',
                            backward: 'S',
                            left: 'A',
                            right: 'D',
                            jump: 'Space',
                            attack: 'LMB',
                            place_block: 'RMB',
                            remove_block: 'LMB',
                            pause: 'Escape',
                            toggle_debug: 'F3',
                        },
                    },
                };
                break;
            case 'SetConfig':
                response = { id, type, ok: true, payload: {} };
                break;
            case 'FetchServers':
                response = {
                    id,
                    type,
                    ok: true,
                    payload: {
                        servers: [
                            {
                                address: '127.0.0.1:7777',
                                name: 'Local Test Server',
                                region: 'Local',
                                tags: ['FFA', 'Vanilla'],
                                players: 3,
                                max_players: 16,
                                protocol_version: 1,
                                is_favorite: false,
                                is_compatible: true,
                            },
                            {
                                address: '192.168.1.100:7777',
                                name: 'LAN Server',
                                region: 'LAN',
                                tags: ['TDM'],
                                players: 8,
                                max_players: 32,
                                protocol_version: 1,
                                is_favorite: true,
                                is_compatible: true,
                            },
                        ],
                    },
                };
                break;
            case 'ToggleFavorite':
                response = {
                    id,
                    type,
                    ok: true,
                    payload: { is_favorite: true },
                };
                break;
            case 'Connect':
                response = { id, type, ok: true, payload: {} };
                // Simulate connection status push
                setTimeout(() => {
                    handlePushEvent('ConnectionStatus', {
                        state: 'connecting',
                        address: payload.address,
                        message: 'Connecting to server...',
                    });
                }, 100);
                setTimeout(() => {
                    handlePushEvent('ConnectionStatus', {
                        state: 'connected',
                        address: payload.address,
                        message: 'Connected!',
                    });
                }, 1500);
                break;
            case 'Quit':
                response = { id, type, ok: true, payload: {} };
                console.log('[Plix] Quit requested - in dev mode, doing nothing');
                break;
            default:
                response = {
                    id,
                    type,
                    ok: false,
                    error: { code: 'EBRG003', message: 'Unknown message type' },
                };
        }

        bridge.handleMessage(JSON.stringify(response));
    }, 50);
}

/**
 * Get current route from hash
 * @returns {string}
 */
function getCurrentRoute() {
    const hash = window.location.hash.slice(1) || '/';
    return hash.startsWith('/') ? hash : '/' + hash;
}

/**
 * Navigate to a route
 * @param {string} route - Route path
 */
function navigate(route) {
    window.location.hash = route;
}

/**
 * Render the current page
 */
function renderPage() {
    const route = getCurrentRoute();
    state.currentRoute = route;

    const PageModule = routes[route];
    const app = document.getElementById('app');

    if (!PageModule) {
        // Unknown route - redirect to main
        navigate('/');
        return;
    }

    // Create page instance and render
    const page = new PageModule({
        navigate,
        bridge,
        state,
    });

    app.innerHTML = page.render();

    // Initialize page (attach event listeners, etc.)
    if (page.init) {
        page.init();
    }
}

/**
 * Handle ESC key for navigation
 * @param {KeyboardEvent} event
 */
function handleKeyDown(event) {
    if (event.key === 'Escape') {
        const route = getCurrentRoute();

        // If on a sub-page, go back to main menu
        if (route !== '/') {
            event.preventDefault();
            navigate('/');
        }
        // If on main menu, let the game handle ESC (close overlay)
    }
}

/**
 * Perform bridge handshake
 */
async function performHandshake() {
    try {
        const result = await bridge.send('Handshake', {
            bridge_version: BRIDGE_VERSION,
        });

        state.playerName = result.display_name || 'Player';
        state.bridgeReady = true;
        console.log('[Plix] Handshake complete, player:', state.playerName);
    } catch (e) {
        console.error('[Plix] Handshake failed:', e);
        state.playerName = 'Player';
        state.bridgeReady = true;
    }
}

/**
 * Initialize the application
 */
async function init() {
    console.log('[Plix] Initializing CEF UI');

    // Register message handler for game engine responses
    if (window.plix) {
        window.plix.onMessage = bridge.handleMessage.bind(bridge);
    }

    // Perform handshake with game engine
    await performHandshake();

    // Set up hash change listener
    window.addEventListener('hashchange', renderPage);

    // Set up ESC key handler
    window.addEventListener('keydown', handleKeyDown);

    // Initial render
    renderPage();
}

// Export for use by pages
export { bridge, state, navigate };

// Start application when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
