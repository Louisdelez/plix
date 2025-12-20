/**
 * CEF Media Embeds Panel (Feature 033)
 *
 * Bridge integration for YouTube/Twitch/Spotify embeds.
 * Uses window.plix.send() and window.plix.onMessage() for communication.
 */

(function() {
    'use strict';

    // === State ===
    const state = {
        visible: false,
        focused: false,
        provider: null,
        embedUrl: null,
        slotState: 'empty', // empty, loading, playing, error
        embedsEnabled: true,
        providersEnabled: {
            youtube: true,
            twitch: true,
            spotify: false
        },
        debugMode: false,
        lastLoadTime: 0,
        rateLimitMs: 2000 // 2 second cooldown
    };

    // === DOM Elements ===
    let panel, urlInput, loadBtn, stopBtn, closeBtn;
    let iframe, chatIframe, errorZone, errorCode, errorMessage;
    let providerLabel, stateLabel, contentArea;
    let debugOverlay, debugProvider, debugUrl, debugFocused, debugVisible;

    // === Request ID Counter ===
    let requestId = 0;
    function nextId() {
        return `emb-${++requestId}-${Date.now()}`;
    }

    // === Bridge Communication ===

    /**
     * Send a message to the Rust game engine via bridge
     * @param {string} type - Message type
     * @param {object} payload - Message payload
     * @returns {Promise<object>} Response promise
     */
    function send(type, payload = {}) {
        return new Promise((resolve, reject) => {
            const id = nextId();
            const message = JSON.stringify({ id, type, payload });

            // Set up response handler
            const handler = (event) => {
                try {
                    const response = JSON.parse(event);
                    if (response.id === id) {
                        window.removeEventListener('plix-message', handler);
                        if (response.ok) {
                            resolve(response.payload || {});
                        } else {
                            reject(response.error || { code: 'UNKNOWN', message: 'Unknown error' });
                        }
                    }
                } catch (e) {
                    // Not our message, ignore
                }
            };

            window.addEventListener('plix-message', handler);

            // Send via bridge
            if (window.plix && window.plix.send) {
                window.plix.send(message);
            } else {
                console.warn('[Embeds] Bridge not available');
                reject({ code: 'EBRG000', message: 'Bridge not available' });
            }

            // Timeout after 5 seconds
            setTimeout(() => {
                window.removeEventListener('plix-message', handler);
                reject({ code: 'ETIMEOUT', message: 'Request timed out' });
            }, 5000);
        });
    }

    /**
     * Handle incoming push messages from Rust
     * @param {string} jsonStr - JSON message string
     */
    function onMessage(jsonStr) {
        try {
            const message = JSON.parse(jsonStr);

            // Dispatch response event for pending requests
            window.dispatchEvent(new CustomEvent('plix-message', { detail: jsonStr }));

            // Handle push events (id is null)
            if (message.id === null) {
                handlePush(message);
            }
        } catch (e) {
            console.error('[Embeds] Failed to parse message:', e);
        }
    }

    /**
     * Handle push events from Rust
     * @param {object} message - Push message
     */
    function handlePush(message) {
        switch (message.type) {
            case 'EmbedState':
                handleEmbedState(message.payload);
                break;
            case 'EmbedError':
                handleEmbedError(message.payload);
                break;
            case 'UiConfig':
                handleUiConfig(message.payload);
                break;
            default:
                // Ignore unknown push types
                break;
        }
    }

    // === Push Handlers ===

    function handleEmbedState(payload) {
        state.visible = payload.visible;
        state.focused = payload.focused;
        state.provider = payload.provider;
        state.embedUrl = payload.embed_url;
        state.slotState = payload.state || 'empty';

        updateUI();

        // Update iframe src if changed
        if (payload.embed_url && iframe.src !== payload.embed_url) {
            iframe.src = payload.embed_url;
        } else if (!payload.embed_url) {
            iframe.src = '';
        }

        // Handle Twitch chat iframe
        if (payload.provider === 'twitch' && payload.chat_url && state.providersEnabled.twitch) {
            chatIframe.src = payload.chat_url;
            chatIframe.classList.remove('hidden');
            contentArea.classList.add('with-chat');
        } else {
            chatIframe.src = '';
            chatIframe.classList.add('hidden');
            contentArea.classList.remove('with-chat');
        }
    }

    function handleEmbedError(payload) {
        showError(payload.code, payload.message);
    }

    function handleUiConfig(payload) {
        if (payload.embedsEnabled !== undefined) {
            state.embedsEnabled = payload.embedsEnabled;
        }
        if (payload.providersEnabled) {
            state.providersEnabled = { ...state.providersEnabled, ...payload.providersEnabled };
        }
        if (payload.debugBridge !== undefined) {
            state.debugMode = payload.debugBridge;
            debugOverlay.classList.toggle('hidden', !state.debugMode);
        }
        updateUI();
    }

    // === UI Updates ===

    function updateUI() {
        // Panel visibility
        panel.classList.toggle('hidden', !state.visible);
        panel.classList.toggle('focused', state.focused);

        // Provider label
        providerLabel.textContent = state.provider ? state.provider.charAt(0).toUpperCase() + state.provider.slice(1) : '';
        providerLabel.className = 'embed-provider' + (state.provider ? ` ${state.provider}` : '');

        // State label
        stateLabel.textContent = state.slotState;
        stateLabel.className = 'embed-state ' + state.slotState;

        // Button states
        stopBtn.disabled = state.slotState === 'empty';
        loadBtn.disabled = !state.embedsEnabled;

        // Debug overlay
        if (state.debugMode) {
            debugProvider.textContent = state.provider || '-';
            debugUrl.textContent = state.embedUrl ? state.embedUrl.substring(0, 50) + '...' : '-';
            debugFocused.textContent = state.focused;
            debugVisible.textContent = state.visible;
        }
    }

    function showError(code, message) {
        errorCode.textContent = code;
        errorMessage.textContent = message;
        errorZone.classList.remove('hidden');

        // Auto-hide after 5 seconds
        setTimeout(() => {
            errorZone.classList.add('hidden');
        }, 5000);
    }

    function hideError() {
        errorZone.classList.add('hidden');
    }

    // === User Actions ===

    async function loadEmbed() {
        const url = urlInput.value.trim();
        if (!url) return;

        // Client-side rate limit check
        const now = Date.now();
        if (now - state.lastLoadTime < state.rateLimitMs) {
            showError('EEMB004', 'Please wait before loading another video');
            return;
        }

        hideError();
        state.lastLoadTime = now;

        // Detect provider from URL
        let provider = 'youtube'; // default
        if (url.includes('twitch.tv') || url.includes('twitch')) {
            provider = 'twitch';
        } else if (url.includes('spotify.com') || url.includes('spotify')) {
            provider = 'spotify';
        }

        try {
            const response = await send('EmbedLoad', {
                provider,
                url_or_id: url
            });

            // Success - iframe will be updated via EmbedState push
            urlInput.value = '';
        } catch (error) {
            showError(error.code || 'UNKNOWN', error.message || 'Failed to load embed');
        }
    }

    async function stopEmbed() {
        hideError();
        try {
            await send('EmbedStop', {});
        } catch (error) {
            showError(error.code || 'UNKNOWN', error.message || 'Failed to stop embed');
        }
    }

    async function closePanel() {
        hideError();
        try {
            await send('EmbedClosePanel', {});
        } catch (error) {
            console.error('[Embeds] Failed to close panel:', error);
        }
    }

    async function requestFocus() {
        if (!state.focused) {
            try {
                await send('EmbedFocus', {});
            } catch (error) {
                console.error('[Embeds] Failed to request focus:', error);
            }
        }
    }

    // === Initialization ===

    function init() {
        // Get DOM elements
        panel = document.getElementById('embed-panel');
        urlInput = document.getElementById('embed-url-input');
        loadBtn = document.getElementById('embed-load-btn');
        stopBtn = document.getElementById('embed-stop-btn');
        closeBtn = document.getElementById('embed-close-btn');
        iframe = document.getElementById('embed-iframe');
        chatIframe = document.getElementById('embed-chat-iframe');
        errorZone = document.getElementById('embed-error');
        errorCode = errorZone.querySelector('.embed-error-code');
        errorMessage = errorZone.querySelector('.embed-error-message');
        providerLabel = document.getElementById('embed-provider');
        stateLabel = document.getElementById('embed-state');
        contentArea = document.querySelector('.embed-content');
        debugOverlay = document.getElementById('embed-debug');
        debugProvider = document.getElementById('debug-provider');
        debugUrl = document.getElementById('debug-url');
        debugFocused = document.getElementById('debug-focused');
        debugVisible = document.getElementById('debug-visible');

        // Event listeners
        loadBtn.addEventListener('click', loadEmbed);
        stopBtn.addEventListener('click', stopEmbed);
        closeBtn.addEventListener('click', closePanel);

        // Enter key to load
        urlInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                loadEmbed();
            }
        });

        // Click on panel to focus
        panel.addEventListener('click', (e) => {
            // Don't request focus if clicking buttons
            if (e.target.tagName !== 'BUTTON' && e.target.tagName !== 'INPUT') {
                requestFocus();
            }
        });

        // Focus input on panel focus
        urlInput.addEventListener('focus', requestFocus);

        // Register message handler
        if (window.plix) {
            window.plix.onMessage = onMessage;
        } else {
            // Create plix namespace if not exists (for testing)
            window.plix = {
                send: (msg) => console.log('[Embeds] Would send:', msg),
                onMessage: onMessage
            };
        }

        // Initial UI state
        updateUI();

        console.log('[Embeds] Initialized');
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
