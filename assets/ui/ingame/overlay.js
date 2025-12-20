/**
 * Plix In-Game Overlay JavaScript (Feature 032)
 *
 * Bridge integration and component management for HUD, Chat, and Scoreboard.
 */

// ============================================================================
// Bridge Communication
// ============================================================================

window.plix = window.plix || {};

/**
 * Send a message to the game engine
 * @param {string} type - Message type
 * @param {object} payload - Message payload
 * @returns {Promise} - Resolves with response or rejects with error
 */
window.plix.send = function(type, payload = {}) {
    return new Promise((resolve, reject) => {
        const id = 'req-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
        const message = JSON.stringify({ id, type, payload });

        // Store callback for response
        window.plix._pending = window.plix._pending || {};
        window.plix._pending[id] = { resolve, reject };

        // Timeout after 5 seconds
        setTimeout(() => {
            if (window.plix._pending[id]) {
                delete window.plix._pending[id];
                reject(new Error('Request timeout'));
            }
        }, 5000);

        // Send to native (CEF will intercept this)
        if (window.plix._nativeSend) {
            window.plix._nativeSend(message);
        } else {
            console.warn('[Bridge] Native send not available:', message);
        }
    });
};

/**
 * Handle incoming message from game engine
 * @param {string} json - JSON message string
 */
window.plix.onMessage = function(json) {
    try {
        const message = JSON.parse(json);

        // Handle response to pending request
        if (message.id && window.plix._pending && window.plix._pending[message.id]) {
            const { resolve, reject } = window.plix._pending[message.id];
            delete window.plix._pending[message.id];

            if (message.ok) {
                resolve(message.payload);
            } else {
                reject(message.error || { code: 'UNKNOWN', message: 'Unknown error' });
            }
            return;
        }

        // Handle push event
        if (message.type && window.plix.handlers[message.type]) {
            window.plix.handlers[message.type](message.payload);
        } else if (message.type) {
            console.warn('[Bridge] Unknown message type:', message.type);
        }
    } catch (e) {
        console.error('[Bridge] Failed to parse message:', e, json);
    }
};

// Message handlers registry
window.plix.handlers = {};

// ============================================================================
// HUD Module
// ============================================================================

const HUD = {
    elements: {
        container: null,
        hpFill: null,
        hpValue: null,
        rtt: null,
        fps: null,
        crosshair: null
    },

    config: {
        enabled: true,
        crosshairEnabled: true
    },

    init() {
        this.elements.container = document.getElementById('hud');
        this.elements.hpFill = document.getElementById('hp-fill');
        this.elements.hpValue = document.getElementById('hp-value');
        this.elements.rtt = document.getElementById('rtt');
        this.elements.fps = document.getElementById('fps');
        this.elements.crosshair = document.getElementById('crosshair');
    },

    update(state) {
        if (!this.config.enabled) return;

        // Update HP bar
        const hpPercent = Math.min(100, Math.max(0, (state.hp / state.max_hp) * 100));
        this.elements.hpFill.style.width = hpPercent + '%';
        this.elements.hpValue.textContent = state.hp;

        // Update HP bar color based on health level
        this.elements.hpFill.classList.remove('low', 'medium');
        if (hpPercent <= 25) {
            this.elements.hpFill.classList.add('low');
        } else if (hpPercent <= 50) {
            this.elements.hpFill.classList.add('medium');
        }

        // Update RTT
        this.elements.rtt.textContent = state.rtt_ms + ' ms';

        // Update FPS (optional)
        if (state.fps !== null && state.fps !== undefined) {
            this.elements.fps.textContent = state.fps + ' FPS';
            this.elements.fps.style.display = '';
        } else {
            this.elements.fps.style.display = 'none';
        }

        // Update crosshair visibility
        if (this.elements.crosshair) {
            this.elements.crosshair.style.display =
                (this.config.crosshairEnabled && state.crosshairVisible !== false) ? '' : 'none';
        }
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
        if (this.elements.container) {
            this.elements.container.style.display = enabled ? '' : 'none';
        }
    },

    setCrosshairEnabled(enabled) {
        this.config.crosshairEnabled = enabled;
        if (this.elements.crosshair) {
            this.elements.crosshair.style.display = enabled ? '' : 'none';
        }
    }
};

// Register HUD message handler
window.plix.handlers.HudState = function(payload) {
    HUD.update(payload);
};

// ============================================================================
// Chat Module
// ============================================================================

const Chat = {
    elements: {
        container: null,
        history: null,
        inputContainer: null,
        input: null
    },

    config: {
        enabled: true,
        maxMessages: 100
    },

    isOpen: false,
    messages: [],

    init() {
        this.elements.container = document.getElementById('chat');
        this.elements.history = document.getElementById('chat-history');
        this.elements.inputContainer = document.getElementById('chat-input-container');
        this.elements.input = document.getElementById('chat-input');

        // Set up input event handlers
        if (this.elements.input) {
            this.elements.input.addEventListener('keydown', (e) => this.handleKeyDown(e));
            this.elements.input.addEventListener('focus', () => this.handleFocus());
            this.elements.input.addEventListener('blur', () => this.handleBlur());
        }
    },

    handleKeyDown(e) {
        if (e.key === 'Enter') {
            e.preventDefault();
            this.send();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            this.close();
        }
    },

    handleFocus() {
        // Notify game that chat is now focused
        window.plix.send('ChatOpen', {}).catch(() => {});
    },

    handleBlur() {
        // Note: Don't close on blur - let game handle focus transitions
    },

    open() {
        if (!this.config.enabled) return;

        this.isOpen = true;
        this.elements.inputContainer.style.display = '';
        this.elements.input.focus();
    },

    close() {
        this.isOpen = false;
        this.elements.inputContainer.style.display = 'none';
        this.elements.input.value = '';
        this.elements.input.blur();

        // Notify game that chat is closed
        window.plix.send('ChatClose', {}).catch(() => {});
    },

    send() {
        const text = this.elements.input.value.trim();
        if (!text) {
            this.close();
            return;
        }

        // Send message to game
        window.plix.send('ChatSend', { text })
            .then(() => {
                this.close();
            })
            .catch((err) => {
                console.error('[Chat] Send failed:', err);
                // Don't close on error, let user retry
            });
    },

    addMessage(msg) {
        // Add to internal array
        this.messages.push(msg);
        if (this.messages.length > this.config.maxMessages) {
            this.messages.shift();
        }

        // Create DOM element
        const el = document.createElement('div');
        el.className = 'chat-message ' + msg.kind;

        const author = document.createElement('span');
        author.className = 'author';
        author.textContent = this.escapeHtml(msg.author) + ':';

        const text = document.createTextNode(' ' + this.escapeHtml(msg.text));

        el.appendChild(author);
        el.appendChild(text);

        // Add to DOM
        this.elements.history.appendChild(el);

        // Scroll to bottom
        this.elements.history.scrollTop = this.elements.history.scrollHeight;

        // Trim DOM if too many messages
        while (this.elements.history.children.length > this.config.maxMessages) {
            this.elements.history.removeChild(this.elements.history.firstChild);
        }
    },

    clear() {
        this.messages = [];
        this.elements.history.innerHTML = '';

        // Notify game
        window.plix.send('ChatClear', {}).catch(() => {});
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
        if (this.elements.container) {
            this.elements.container.style.display = enabled ? '' : 'none';
        }
    },

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
};

// Register Chat message handlers
window.plix.handlers.ChatMessage = function(payload) {
    Chat.addMessage(payload);
};

// ============================================================================
// Toast Module
// ============================================================================

const Toast = {
    elements: {
        container: null
    },

    config: {
        maxToasts: 3,
        duration: 3000 // 3 seconds
    },

    toasts: [],

    init() {
        this.elements.container = document.getElementById('toasts');
    },

    show(msg) {
        // Create toast element
        const el = document.createElement('div');
        el.className = 'toast';

        const author = document.createElement('span');
        author.className = 'author';
        author.textContent = Chat.escapeHtml(msg.author) + ':';

        const text = document.createTextNode(' ' + Chat.escapeHtml(msg.text));

        el.appendChild(author);
        el.appendChild(text);

        // Track toast
        const toast = { el, timeout: null };
        this.toasts.push(toast);

        // Remove oldest if at limit
        if (this.toasts.length > this.config.maxToasts) {
            const oldest = this.toasts.shift();
            this.dismiss(oldest);
        }

        // Add to DOM
        this.elements.container.appendChild(el);

        // Auto-dismiss after duration
        toast.timeout = setTimeout(() => {
            this.dismiss(toast);
        }, this.config.duration);
    },

    dismiss(toast) {
        if (toast.timeout) {
            clearTimeout(toast.timeout);
            toast.timeout = null;
        }

        if (toast.el && toast.el.parentNode) {
            toast.el.classList.add('fade-out');
            setTimeout(() => {
                if (toast.el.parentNode) {
                    toast.el.parentNode.removeChild(toast.el);
                }
            }, 300);
        }

        // Remove from array
        const index = this.toasts.indexOf(toast);
        if (index !== -1) {
            this.toasts.splice(index, 1);
        }
    },

    clearAll() {
        for (const toast of this.toasts) {
            this.dismiss(toast);
        }
        this.toasts = [];
    }
};

// Register Toast message handler
window.plix.handlers.ChatToast = function(payload) {
    Toast.show(payload);
};

// ============================================================================
// Scoreboard Module
// ============================================================================

const Scoreboard = {
    elements: {
        container: null,
        serverName: null,
        body: null
    },

    config: {
        enabled: true
    },

    isVisible: false,

    init() {
        this.elements.container = document.getElementById('scoreboard');
        this.elements.serverName = document.getElementById('server-name');
        this.elements.body = document.getElementById('scoreboard-body');
    },

    show() {
        if (!this.config.enabled) return;

        this.isVisible = true;
        this.elements.container.style.display = '';
    },

    hide() {
        this.isVisible = false;
        this.elements.container.style.display = 'none';
    },

    update(state) {
        if (!this.config.enabled || !this.isVisible) return;

        // Update server name
        this.elements.serverName.textContent = state.server_name || 'Server';

        // Clear existing rows
        this.elements.body.innerHTML = '';

        // Add rows
        for (const row of state.rows) {
            const tr = document.createElement('tr');

            // Team class if applicable
            if (row.team) {
                tr.className = 'team-' + row.team;
            }

            // Player name
            const tdName = document.createElement('td');
            tdName.className = 'player-name';
            tdName.textContent = this.escapeHtml(row.name);
            tr.appendChild(tdName);

            // Kills
            const tdKills = document.createElement('td');
            tdKills.textContent = row.kills !== null && row.kills !== undefined ? row.kills : '-';
            tr.appendChild(tdKills);

            // Deaths
            const tdDeaths = document.createElement('td');
            tdDeaths.textContent = row.deaths !== null && row.deaths !== undefined ? row.deaths : '-';
            tr.appendChild(tdDeaths);

            // Ping
            const tdPing = document.createElement('td');
            tdPing.textContent = row.ping_ms + ' ms';
            tr.appendChild(tdPing);

            this.elements.body.appendChild(tr);
        }
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
    },

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
};

// Register Scoreboard message handler
window.plix.handlers.ScoreboardState = function(payload) {
    Scoreboard.update(payload);
};

// ============================================================================
// Subtitles Module (Feature 042)
// ============================================================================

const Subtitles = {
    elements: {
        container: null
    },

    config: {
        enabled: false,
        size: 'medium',
        backgroundOpacity: 75
    },

    subtitles: new Map(), // id -> { el, timeout }

    init() {
        this.elements.container = document.getElementById('subtitles');
    },

    show(payload) {
        if (!this.config.enabled || !this.elements.container) return;

        const { id, text, duration_ms } = payload;

        // If subtitle with this ID already exists, update it
        if (this.subtitles.has(id)) {
            this.dismiss(id);
        }

        // Create subtitle element
        const el = document.createElement('div');
        el.className = `subtitle-entry size-${this.config.size}`;
        el.textContent = text;

        // Apply background opacity
        const bgOpacity = this.config.backgroundOpacity / 100;
        el.style.setProperty('--subtitle-bg-opacity', bgOpacity);

        // Add to DOM
        this.elements.container.appendChild(el);

        // Track subtitle
        const timeout = setTimeout(() => {
            this.dismiss(id);
        }, duration_ms);

        this.subtitles.set(id, { el, timeout });

        // Enforce max 3 subtitles
        if (this.subtitles.size > 3) {
            // Find and remove oldest (first entry in Map)
            const oldestId = this.subtitles.keys().next().value;
            this.dismiss(oldestId);
        }
    },

    dismiss(id) {
        const subtitle = this.subtitles.get(id);
        if (!subtitle) return;

        // Clear timeout
        if (subtitle.timeout) {
            clearTimeout(subtitle.timeout);
        }

        // Fade out animation
        subtitle.el.classList.add('fade-out');
        setTimeout(() => {
            if (subtitle.el.parentNode) {
                subtitle.el.parentNode.removeChild(subtitle.el);
            }
        }, 300);

        this.subtitles.delete(id);
    },

    clear() {
        for (const [id, subtitle] of this.subtitles) {
            if (subtitle.timeout) {
                clearTimeout(subtitle.timeout);
            }
            if (subtitle.el.parentNode) {
                subtitle.el.parentNode.removeChild(subtitle.el);
            }
        }
        this.subtitles.clear();
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
        if (!enabled) {
            this.clear();
        }
    },

    setSize(size) {
        this.config.size = size;
        // Update existing subtitles
        for (const [id, subtitle] of this.subtitles) {
            subtitle.el.classList.remove('size-small', 'size-medium', 'size-large');
            subtitle.el.classList.add(`size-${size}`);
        }
    },

    setBackgroundOpacity(opacity) {
        this.config.backgroundOpacity = opacity;
        // Update existing subtitles
        const bgOpacity = opacity / 100;
        for (const [id, subtitle] of this.subtitles) {
            subtitle.el.style.setProperty('--subtitle-bg-opacity', bgOpacity);
        }
    }
};

// Register Subtitle message handlers
window.plix.handlers.SubtitleShow = function(payload) {
    Subtitles.show(payload);
};

window.plix.handlers.SubtitleClear = function(payload) {
    Subtitles.clear();
};

// ============================================================================
// Accessibility Settings Handler (Feature 042)
// ============================================================================

window.plix.handlers.AccessibilitySettings = function(payload) {
    // Apply subtitle settings
    Subtitles.setEnabled(payload.subtitles_enabled || false);
    Subtitles.setSize(payload.subtitle_size || 'medium');
    Subtitles.setBackgroundOpacity(payload.subtitle_bg_opacity || 75);

    // Apply visual accessibility (if body element available)
    if (payload.high_contrast) {
        document.documentElement.classList.add('high-contrast');
    } else {
        document.documentElement.classList.remove('high-contrast');
    }

    // Apply colorblind preset
    document.documentElement.classList.remove(
        'colorblind-protanopia',
        'colorblind-deuteranopia',
        'colorblind-tritanopia'
    );
    if (payload.colorblind_preset && payload.colorblind_preset !== 'none') {
        document.documentElement.classList.add(`colorblind-${payload.colorblind_preset}`);
    }

    // Apply UI scale
    if (payload.ui_scale) {
        const scaleFactor = payload.ui_scale / 100;
        document.documentElement.style.setProperty('--ui-scale', scaleFactor);
    }

    console.log('[Overlay] Accessibility settings applied:', payload);
};

// ============================================================================
// Dungeon HUD Module (Feature 043)
// ============================================================================

const DungeonHUD = {
    elements: {
        container: null,
        dungeonName: null,
        objective: null,
        bossHpContainer: null,
        bossName: null,
        bossHpFill: null,
        bossHpText: null,
        chestContainer: null
    },

    config: {
        enabled: true
    },

    state: {
        inDungeon: false,
        dungeonId: null,
        bossAlive: true,
        bossHpPercent: 100,
        chestAvailable: false
    },

    init() {
        this.elements.container = document.getElementById('dungeon-hud');
        this.elements.dungeonName = document.getElementById('dungeon-name');
        this.elements.objective = document.getElementById('dungeon-objective');
        this.elements.bossHpContainer = document.getElementById('dungeon-boss-hp-container');
        this.elements.bossName = document.getElementById('boss-name');
        this.elements.bossHpFill = document.getElementById('boss-hp-fill');
        this.elements.bossHpText = document.getElementById('boss-hp-text');
        this.elements.chestContainer = document.getElementById('dungeon-chest-container');
    },

    show(payload) {
        if (!this.config.enabled || !this.elements.container) return;

        this.state.inDungeon = true;
        this.state.dungeonId = payload.dungeon_id;

        // Set dungeon info
        this.elements.dungeonName.textContent = payload.display_name || 'Unknown Dungeon';
        this.elements.objective.textContent = payload.objective || 'Defeat the boss';

        // Reset state
        this.state.bossAlive = true;
        this.state.chestAvailable = false;
        this.updateBossHP(100);
        this.elements.chestContainer.style.display = 'none';
        this.elements.bossHpContainer.style.display = '';

        // Show container
        this.elements.container.style.display = '';
    },

    hide() {
        this.state.inDungeon = false;
        this.state.dungeonId = null;
        if (this.elements.container) {
            this.elements.container.style.display = 'none';
        }
    },

    updateState(payload) {
        if (!this.config.enabled || !this.state.inDungeon) return;
        if (payload.dungeon_id !== this.state.dungeonId) return;

        this.state.bossAlive = payload.boss_alive;
        this.state.chestAvailable = payload.chest_available;

        // Update boss HP display
        if (payload.boss_hp_percent !== null && payload.boss_hp_percent !== undefined) {
            this.updateBossHP(payload.boss_hp_percent * 100);
        }

        // Show/hide boss HP bar based on boss alive state
        if (this.elements.bossHpContainer) {
            this.elements.bossHpContainer.style.display = this.state.bossAlive ? '' : 'none';
        }

        // Show/hide chest available indicator
        if (this.elements.chestContainer) {
            this.elements.chestContainer.style.display = this.state.chestAvailable ? '' : 'none';
        }

        // Update objective text based on state
        if (this.elements.objective) {
            if (this.state.chestAvailable) {
                this.elements.objective.textContent = 'Open the reward chest!';
            } else if (!this.state.bossAlive) {
                this.elements.objective.textContent = 'Boss defeated! Waiting for chest...';
            } else {
                this.elements.objective.textContent = 'Defeat the boss';
            }
        }
    },

    updateBossHP(percent) {
        this.state.bossHpPercent = Math.max(0, Math.min(100, percent));

        if (this.elements.bossHpFill) {
            this.elements.bossHpFill.style.width = this.state.bossHpPercent + '%';
        }

        if (this.elements.bossHpText) {
            this.elements.bossHpText.textContent = Math.round(this.state.bossHpPercent) + '%';
        }
    },

    onChestAvailable(payload) {
        if (!this.config.enabled || !this.state.inDungeon) return;
        if (payload.dungeon_id !== this.state.dungeonId) return;

        this.state.chestAvailable = true;
        if (this.elements.chestContainer) {
            this.elements.chestContainer.style.display = '';
        }
        if (this.elements.objective) {
            this.elements.objective.textContent = 'Open the reward chest!';
        }
    },

    onCompleted(payload) {
        if (!this.config.enabled || !this.state.inDungeon) return;
        if (payload.dungeon_id !== this.state.dungeonId) return;

        // Show completion message
        if (this.elements.objective) {
            this.elements.objective.textContent = 'Dungeon completed!';
        }
        if (this.elements.chestContainer) {
            this.elements.chestContainer.style.display = 'none';
        }

        // Toast the rewards
        if (payload.rewards && payload.rewards.length > 0) {
            const rewardText = payload.rewards.map(r => `${r[1]}x ${r[0]}`).join(', ');
            Toast.show({ author: 'Dungeon', text: `Rewards: ${rewardText}` });
        }
        if (payload.xp_gained > 0) {
            Toast.show({ author: 'Dungeon', text: `+${payload.xp_gained} XP` });
        }

        // Hide HUD after delay
        setTimeout(() => {
            this.hide();
        }, 3000);
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
        if (!enabled && this.elements.container) {
            this.elements.container.style.display = 'none';
        }
    }
};

// Register DungeonHUD message handlers
window.plix.handlers.DungeonEntered = function(payload) {
    DungeonHUD.show(payload);
};

window.plix.handlers.DungeonStateUpdate = function(payload) {
    DungeonHUD.updateState(payload);
};

window.plix.handlers.ChestAvailable = function(payload) {
    DungeonHUD.onChestAvailable(payload);
};

window.plix.handlers.DungeonCompleted = function(payload) {
    DungeonHUD.onCompleted(payload);
};

// ============================================================================
// Dialogue Module (Feature 043)
// ============================================================================

const Dialogue = {
    elements: {
        container: null,
        npcName: null,
        lines: null,
        options: null
    },

    config: {
        enabled: true
    },

    state: {
        isOpen: false,
        npcId: null
    },

    init() {
        this.elements.container = document.getElementById('dialogue-panel');
        this.elements.npcName = document.getElementById('dialogue-npc-name');
        this.elements.lines = document.getElementById('dialogue-lines');
        this.elements.options = document.getElementById('dialogue-options');
    },

    show(payload) {
        if (!this.config.enabled || !this.elements.container) return;

        this.state.isOpen = true;
        this.state.npcId = payload.npc_id;

        // Set NPC name
        this.elements.npcName.textContent = payload.npc_name || 'Unknown';

        // Clear and populate lines
        this.elements.lines.innerHTML = '';
        for (const line of (payload.lines || [])) {
            const lineEl = document.createElement('div');
            lineEl.className = 'dialogue-line';
            lineEl.textContent = line;
            this.elements.lines.appendChild(lineEl);
        }

        // Clear and populate options
        this.elements.options.innerHTML = '';
        for (const option of (payload.options || [])) {
            const optionEl = document.createElement('button');
            optionEl.className = 'dialogue-option';
            optionEl.textContent = option.text;
            optionEl.dataset.optionId = option.option_id;

            // Add styling class based on option type
            switch (option.option_type) {
                case 'AcceptQuest':
                    optionEl.classList.add('accept-quest');
                    break;
                case 'DeclineQuest':
                    optionEl.classList.add('decline-quest');
                    break;
                case 'Close':
                    optionEl.classList.add('close');
                    break;
            }

            // Click handler
            optionEl.addEventListener('click', () => {
                this.selectOption(option.option_id);
            });

            this.elements.options.appendChild(optionEl);
        }

        // Show container
        this.elements.container.style.display = '';
    },

    hide() {
        this.state.isOpen = false;
        this.state.npcId = null;
        if (this.elements.container) {
            this.elements.container.style.display = 'none';
        }
    },

    selectOption(optionId) {
        if (!this.state.isOpen || !this.state.npcId) return;

        // Send response to game
        window.plix.send('DialogueResponse', {
            npc_id: this.state.npcId,
            option_id: optionId
        }).then(() => {
            this.hide();
        }).catch((err) => {
            console.error('[Dialogue] Response failed:', err);
            // Hide anyway on error
            this.hide();
        });
    },

    setEnabled(enabled) {
        this.config.enabled = enabled;
        if (!enabled) {
            this.hide();
        }
    }
};

// Register Dialogue message handler
window.plix.handlers.DialogueShow = function(payload) {
    Dialogue.show(payload);
};

// ============================================================================
// UI Config Handler
// ============================================================================

window.plix.handlers.UiConfig = function(payload) {
    // Apply HUD config
    HUD.setEnabled(payload.cefHudEnabled !== false);
    HUD.setCrosshairEnabled(payload.cefCrosshairEnabled !== false);

    // Apply Chat config
    Chat.setEnabled(payload.cefChatEnabled !== false);

    // Apply Scoreboard config
    Scoreboard.setEnabled(payload.cefScoreboardEnabled !== false);

    console.log('[Overlay] UI config applied:', payload);
};

// ============================================================================
// Initialization
// ============================================================================

document.addEventListener('DOMContentLoaded', function() {
    console.log('[Overlay] Initializing...');

    HUD.init();
    Chat.init();
    Toast.init();
    Scoreboard.init();
    Subtitles.init();
    DungeonHUD.init();
    Dialogue.init();

    console.log('[Overlay] Ready');

    // Signal ready to game
    if (window.plix._onReady) {
        window.plix._onReady();
    }
});

// Export modules for debugging
window.plix.HUD = HUD;
window.plix.Chat = Chat;
window.plix.Toast = Toast;
window.plix.Scoreboard = Scoreboard;
window.plix.Subtitles = Subtitles;
window.plix.DungeonHUD = DungeonHUD;
window.plix.Dialogue = Dialogue;
