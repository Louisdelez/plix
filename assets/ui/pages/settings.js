/**
 * Settings Page (Feature 031, Feature 042)
 *
 * Displays and allows modification of game settings.
 * Settings: sensitivity, FOV, fullscreen, audio mute, keybinds
 * Accessibility: UI scale, high contrast, colorblind presets, subtitles
 */

// Settings constraints (must match Rust constants)
const SENSITIVITY_MIN = 0.0001;
const SENSITIVITY_MAX = 0.01;
const FOV_MIN = 60;
const FOV_MAX = 110;

// Accessibility constraints (Feature 042)
const UI_SCALE_MIN = 75;
const UI_SCALE_MAX = 150;
const UI_SCALE_DEFAULT = 100;

const COLORBLIND_PRESETS = [
    { value: 'none', label: 'None' },
    { value: 'protanopia', label: 'Protanopia (Red-blind)' },
    { value: 'deuteranopia', label: 'Deuteranopia (Green-blind)' },
    { value: 'tritanopia', label: 'Tritanopia (Blue-blind)' },
];

const SUBTITLE_SIZES = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
];

export class SettingsPage {
    /**
     * @param {object} context - Application context
     */
    constructor(context) {
        this.navigate = context.navigate;
        this.bridge = context.bridge;
        this.state = context.state;

        /** @type {object|null} Current config */
        this.config = null;

        /** @type {object|null} Modified config (pending save) */
        this.pendingConfig = null;

        /** @type {string|null} Currently editing keybind */
        this.editingKeybind = null;

        /** @type {string|null} Error message */
        this.error = null;

        /** @type {string|null} Success message */
        this.success = null;
    }

    render() {
        return `
            <div class="page">
                <div class="page-header">
                    <h1 class="page-title">Settings</h1>
                </div>

                <div id="settings-content">
                    <div class="loading">
                        <div class="loading-spinner"></div>
                        <div class="loading-text">Loading settings...</div>
                    </div>
                </div>

                <div class="action-bar">
                    <button class="btn" id="btn-cancel">Cancel</button>
                    <button class="btn btn-primary" id="btn-save" disabled>Save</button>
                </div>
            </div>
        `;
    }

    async init() {
        // Set up cancel button
        document.getElementById('btn-cancel')?.addEventListener('click', () => {
            this.navigate('/');
        });

        // Set up save button
        document.getElementById('btn-save')?.addEventListener('click', () => {
            this.saveConfig();
        });

        // Load config
        await this.loadConfig();
    }

    async loadConfig() {
        try {
            const payload = await this.bridge.send('GetConfig', {});
            this.config = payload;
            this.pendingConfig = JSON.parse(JSON.stringify(payload));
            this.renderSettings();
        } catch (e) {
            this.showError('Failed to load settings: ' + e.message);
        }
    }

    renderSettings() {
        const content = document.getElementById('settings-content');
        if (!content || !this.config) return;

        content.innerHTML = `
            <div id="messages"></div>

            <div class="form-group">
                <label class="form-label">Mouse Sensitivity</label>
                <div class="range-container">
                    <input type="range" class="range-input" id="sensitivity"
                        min="${SENSITIVITY_MIN}" max="${SENSITIVITY_MAX}" step="0.0001"
                        value="${this.pendingConfig.sensitivity}">
                    <span class="range-value" id="sensitivity-value">
                        ${formatSensitivity(this.pendingConfig.sensitivity)}
                    </span>
                </div>
            </div>

            <div class="form-group">
                <label class="form-label">Field of View</label>
                <div class="range-container">
                    <input type="range" class="range-input" id="fov"
                        min="${FOV_MIN}" max="${FOV_MAX}" step="1"
                        value="${this.pendingConfig.fov_degrees}">
                    <span class="range-value" id="fov-value">
                        ${Math.round(this.pendingConfig.fov_degrees)}°
                    </span>
                </div>
            </div>

            <div class="form-group">
                <label class="checkbox-container">
                    <input type="checkbox" class="checkbox-input" id="fullscreen"
                        ${this.pendingConfig.fullscreen ? 'checked' : ''}>
                    <span class="checkbox-label">Fullscreen</span>
                </label>
            </div>

            <div class="form-group">
                <label class="checkbox-container">
                    <input type="checkbox" class="checkbox-input" id="audio-muted"
                        ${this.pendingConfig.audio_muted ? 'checked' : ''}>
                    <span class="checkbox-label">Mute Audio</span>
                </label>
            </div>

            <div class="form-group">
                <label class="form-label">Key Bindings</label>
                <div id="keybinds-container">
                    ${this.renderKeybinds()}
                </div>
            </div>

            ${this.renderAccessibilitySettings()}
        `;

        this.attachSettingsListeners();
    }

    renderKeybinds() {
        const keybinds = this.pendingConfig.keybinds || {};
        const actions = [
            { key: 'forward', label: 'Forward' },
            { key: 'backward', label: 'Backward' },
            { key: 'left', label: 'Left' },
            { key: 'right', label: 'Right' },
            { key: 'jump', label: 'Jump' },
            { key: 'attack', label: 'Attack' },
            { key: 'place_block', label: 'Place Block' },
            { key: 'remove_block', label: 'Remove Block' },
            { key: 'pause', label: 'Pause' },
            { key: 'toggle_debug', label: 'Debug Overlay' },
        ];

        return actions
            .map(
                ({ key, label }) => `
            <div class="settings-row">
                <span class="settings-label">${label}</span>
                <button class="keybind-btn ${this.editingKeybind === key ? 'editing' : ''}"
                    data-action="${key}" id="keybind-${key}">
                    ${escapeHtml(keybinds[key] || '?')}
                </button>
            </div>
        `
            )
            .join('');
    }

    /**
     * Render accessibility settings section (Feature 042)
     */
    renderAccessibilitySettings() {
        const accessibility = this.pendingConfig.accessibility || {
            ui_scale: UI_SCALE_DEFAULT,
            high_contrast: false,
            colorblind_preset: 'none',
            subtitles: {
                enabled: false,
                size: 'medium',
                background_opacity: 75,
            },
        };

        const colorblindOptions = COLORBLIND_PRESETS.map(
            (p) =>
                `<option value="${p.value}" ${accessibility.colorblind_preset === p.value ? 'selected' : ''}>${p.label}</option>`
        ).join('');

        const subtitleSizeOptions = SUBTITLE_SIZES.map(
            (s) =>
                `<option value="${s.value}" ${accessibility.subtitles?.size === s.value ? 'selected' : ''}>${s.label}</option>`
        ).join('');

        return `
            <div class="form-section">
                <h3 class="form-section-title">Accessibility</h3>

                <div class="form-group">
                    <label class="form-label">UI Scale</label>
                    <div class="range-container">
                        <input type="range" class="range-input" id="ui-scale"
                            min="${UI_SCALE_MIN}" max="${UI_SCALE_MAX}" step="5"
                            value="${accessibility.ui_scale}">
                        <span class="range-value" id="ui-scale-value">
                            ${accessibility.ui_scale}%
                        </span>
                    </div>
                </div>

                <div class="form-group">
                    <label class="checkbox-container">
                        <input type="checkbox" class="checkbox-input" id="high-contrast"
                            ${accessibility.high_contrast ? 'checked' : ''}>
                        <span class="checkbox-label">High Contrast Mode</span>
                    </label>
                </div>

                <div class="form-group">
                    <label class="form-label">Colorblind Mode</label>
                    <select id="colorblind-preset" class="select-input">
                        ${colorblindOptions}
                    </select>
                </div>
            </div>

            <div class="form-section">
                <h3 class="form-section-title">Subtitles</h3>

                <div class="form-group">
                    <label class="checkbox-container">
                        <input type="checkbox" class="checkbox-input" id="subtitles-enabled"
                            ${accessibility.subtitles?.enabled ? 'checked' : ''}>
                        <span class="checkbox-label">Enable Subtitles</span>
                    </label>
                </div>

                <div class="form-group">
                    <label class="form-label">Subtitle Size</label>
                    <select id="subtitle-size" class="select-input">
                        ${subtitleSizeOptions}
                    </select>
                </div>

                <div class="form-group">
                    <label class="form-label">Subtitle Background Opacity</label>
                    <div class="range-container">
                        <input type="range" class="range-input" id="subtitle-opacity"
                            min="0" max="100" step="5"
                            value="${accessibility.subtitles?.background_opacity ?? 75}">
                        <span class="range-value" id="subtitle-opacity-value">
                            ${accessibility.subtitles?.background_opacity ?? 75}%
                        </span>
                    </div>
                </div>
            </div>
        `;
    }

    attachSettingsListeners() {
        // Sensitivity slider
        const sensitivity = document.getElementById('sensitivity');
        sensitivity?.addEventListener('input', (e) => {
            const value = parseFloat(e.target.value);
            this.pendingConfig.sensitivity = value;
            document.getElementById('sensitivity-value').textContent =
                formatSensitivity(value);
            this.markDirty();
        });

        // FOV slider
        const fov = document.getElementById('fov');
        fov?.addEventListener('input', (e) => {
            const value = parseInt(e.target.value, 10);
            this.pendingConfig.fov_degrees = value;
            document.getElementById('fov-value').textContent = `${value}°`;
            this.markDirty();
        });

        // Fullscreen checkbox
        const fullscreen = document.getElementById('fullscreen');
        fullscreen?.addEventListener('change', (e) => {
            this.pendingConfig.fullscreen = e.target.checked;
            this.markDirty();
        });

        // Audio muted checkbox
        const audioMuted = document.getElementById('audio-muted');
        audioMuted?.addEventListener('change', (e) => {
            this.pendingConfig.audio_muted = e.target.checked;
            this.markDirty();
        });

        // Keybind buttons
        document.querySelectorAll('.keybind-btn').forEach((btn) => {
            btn.addEventListener('click', (e) => {
                const action = e.target.dataset.action;
                this.startKeybindEdit(action);
            });
        });

        // Global keydown for keybind capture
        this._keyHandler = (e) => this.handleKeybindCapture(e);
        window.addEventListener('keydown', this._keyHandler);

        // Global mousedown for keybind capture
        this._mouseHandler = (e) => this.handleMouseCapture(e);
        window.addEventListener('mousedown', this._mouseHandler);

        // === Accessibility settings (Feature 042) ===

        // UI Scale slider
        const uiScale = document.getElementById('ui-scale');
        uiScale?.addEventListener('input', (e) => {
            const value = parseInt(e.target.value, 10);
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = {};
            }
            this.pendingConfig.accessibility.ui_scale = value;
            document.getElementById('ui-scale-value').textContent = `${value}%`;
            // Apply live preview
            this.applyUiScale(value);
            this.markDirty();
        });

        // High contrast toggle
        const highContrast = document.getElementById('high-contrast');
        highContrast?.addEventListener('change', (e) => {
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = {};
            }
            this.pendingConfig.accessibility.high_contrast = e.target.checked;
            // Apply live preview
            this.applyHighContrast(e.target.checked);
            this.markDirty();
        });

        // Colorblind preset
        const colorblindPreset = document.getElementById('colorblind-preset');
        colorblindPreset?.addEventListener('change', (e) => {
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = {};
            }
            this.pendingConfig.accessibility.colorblind_preset = e.target.value;
            // Apply live preview
            this.applyColorblindPreset(e.target.value);
            this.markDirty();
        });

        // Subtitles enabled
        const subtitlesEnabled = document.getElementById('subtitles-enabled');
        subtitlesEnabled?.addEventListener('change', (e) => {
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = { subtitles: {} };
            }
            if (!this.pendingConfig.accessibility.subtitles) {
                this.pendingConfig.accessibility.subtitles = {};
            }
            this.pendingConfig.accessibility.subtitles.enabled = e.target.checked;
            this.markDirty();
        });

        // Subtitle size
        const subtitleSize = document.getElementById('subtitle-size');
        subtitleSize?.addEventListener('change', (e) => {
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = { subtitles: {} };
            }
            if (!this.pendingConfig.accessibility.subtitles) {
                this.pendingConfig.accessibility.subtitles = {};
            }
            this.pendingConfig.accessibility.subtitles.size = e.target.value;
            this.markDirty();
        });

        // Subtitle opacity
        const subtitleOpacity = document.getElementById('subtitle-opacity');
        subtitleOpacity?.addEventListener('input', (e) => {
            const value = parseInt(e.target.value, 10);
            if (!this.pendingConfig.accessibility) {
                this.pendingConfig.accessibility = { subtitles: {} };
            }
            if (!this.pendingConfig.accessibility.subtitles) {
                this.pendingConfig.accessibility.subtitles = {};
            }
            this.pendingConfig.accessibility.subtitles.background_opacity = value;
            document.getElementById('subtitle-opacity-value').textContent = `${value}%`;
            this.markDirty();
        });
    }

    /**
     * Apply UI scale live preview (Feature 042)
     * @param {number} scale - Scale percentage (75-150)
     */
    applyUiScale(scale) {
        const scaleFactor = scale / 100;
        document.documentElement.style.setProperty('--ui-scale', scaleFactor);
    }

    /**
     * Apply high contrast mode live preview (Feature 042)
     * @param {boolean} enabled
     */
    applyHighContrast(enabled) {
        if (enabled) {
            document.documentElement.classList.add('high-contrast');
        } else {
            document.documentElement.classList.remove('high-contrast');
        }
    }

    /**
     * Apply colorblind preset live preview (Feature 042)
     * @param {string} preset
     */
    applyColorblindPreset(preset) {
        // Remove all colorblind classes
        document.documentElement.classList.remove(
            'colorblind-protanopia',
            'colorblind-deuteranopia',
            'colorblind-tritanopia'
        );
        // Add new class if not 'none'
        if (preset && preset !== 'none') {
            document.documentElement.classList.add(`colorblind-${preset}`);
        }
    }

    startKeybindEdit(action) {
        // If already editing, cancel
        if (this.editingKeybind === action) {
            this.cancelKeybindEdit();
            return;
        }

        // Cancel any previous edit
        this.cancelKeybindEdit();

        this.editingKeybind = action;
        const btn = document.getElementById(`keybind-${action}`);
        if (btn) {
            btn.classList.add('editing');
            btn.textContent = 'Press key...';
        }
    }

    cancelKeybindEdit() {
        if (!this.editingKeybind) return;

        const btn = document.getElementById(`keybind-${this.editingKeybind}`);
        if (btn) {
            btn.classList.remove('editing');
            btn.textContent =
                this.pendingConfig.keybinds[this.editingKeybind] || '?';
        }
        this.editingKeybind = null;
    }

    handleKeybindCapture(e) {
        if (!this.editingKeybind) return;

        // ESC cancels edit
        if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            this.cancelKeybindEdit();
            return;
        }

        e.preventDefault();
        e.stopPropagation();

        const keyName = getKeyName(e);
        if (!keyName) return;

        this.assignKeybind(this.editingKeybind, keyName);
    }

    handleMouseCapture(e) {
        if (!this.editingKeybind) return;

        // Only capture mouse buttons 0-2
        if (e.button > 2) return;

        // Ignore clicks on keybind buttons (let them toggle)
        if (e.target.classList.contains('keybind-btn')) return;

        e.preventDefault();
        e.stopPropagation();

        const buttonName = getMouseButtonName(e.button);
        this.assignKeybind(this.editingKeybind, buttonName);
    }

    assignKeybind(action, keyName) {
        // Check for conflict
        const conflictAction = Object.entries(this.pendingConfig.keybinds).find(
            ([a, k]) => a !== action && k === keyName
        );

        if (conflictAction) {
            // Swap keybinds
            const [conflictKey] = conflictAction;
            const oldKey = this.pendingConfig.keybinds[action];
            this.pendingConfig.keybinds[conflictKey] = oldKey;
            this.pendingConfig.keybinds[action] = keyName;

            // Update both buttons
            const conflictBtn = document.getElementById(`keybind-${conflictKey}`);
            if (conflictBtn) {
                conflictBtn.textContent = oldKey;
            }
        } else {
            this.pendingConfig.keybinds[action] = keyName;
        }

        // Update button
        const btn = document.getElementById(`keybind-${action}`);
        if (btn) {
            btn.classList.remove('editing');
            btn.textContent = keyName;
        }

        this.editingKeybind = null;
        this.markDirty();
    }

    markDirty() {
        const saveBtn = document.getElementById('btn-save');
        if (saveBtn) {
            saveBtn.disabled = false;
        }
        this.clearMessages();
    }

    async saveConfig() {
        const saveBtn = document.getElementById('btn-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.classList.add('btn-loading');
        }

        try {
            await this.bridge.send('SetConfig', this.pendingConfig);
            this.config = JSON.parse(JSON.stringify(this.pendingConfig));
            this.showSuccess('Settings saved successfully');
        } catch (e) {
            this.showError('Failed to save settings: ' + e.message);
            if (saveBtn) {
                saveBtn.disabled = false;
            }
        } finally {
            if (saveBtn) {
                saveBtn.classList.remove('btn-loading');
            }
        }
    }

    showError(message) {
        this.error = message;
        this.success = null;
        this.updateMessages();
    }

    showSuccess(message) {
        this.success = message;
        this.error = null;
        this.updateMessages();
    }

    clearMessages() {
        this.error = null;
        this.success = null;
        this.updateMessages();
    }

    updateMessages() {
        const container = document.getElementById('messages');
        if (!container) return;

        if (this.error) {
            container.innerHTML = `<div class="error-message">${escapeHtml(this.error)}</div>`;
        } else if (this.success) {
            container.innerHTML = `<div class="success-message">${escapeHtml(this.success)}</div>`;
        } else {
            container.innerHTML = '';
        }
    }
}

/**
 * Format sensitivity for display
 * @param {number} value
 * @returns {string}
 */
function formatSensitivity(value) {
    return value.toFixed(4);
}

/**
 * Get key name from keyboard event
 * @param {KeyboardEvent} e
 * @returns {string|null}
 */
function getKeyName(e) {
    // Special keys
    const specialKeys = {
        Space: 'Space',
        Enter: 'Enter',
        Tab: 'Tab',
        Backspace: 'Backspace',
        ControlLeft: 'Ctrl',
        ControlRight: 'Ctrl',
        ShiftLeft: 'Shift',
        ShiftRight: 'Shift',
        AltLeft: 'Alt',
        AltRight: 'Alt',
        ArrowUp: 'Up',
        ArrowDown: 'Down',
        ArrowLeft: 'Left Arrow',
        ArrowRight: 'Right Arrow',
    };

    if (specialKeys[e.code]) {
        return specialKeys[e.code];
    }

    // Letters (KeyA -> A)
    if (e.code.startsWith('Key')) {
        return e.code.slice(3);
    }

    // Digits (Digit1 -> 1)
    if (e.code.startsWith('Digit')) {
        return e.code.slice(5);
    }

    // Function keys (F1-F12)
    if (/^F\d+$/.test(e.code)) {
        return e.code;
    }

    return null;
}

/**
 * Get mouse button name
 * @param {number} button
 * @returns {string}
 */
function getMouseButtonName(button) {
    switch (button) {
        case 0:
            return 'LMB';
        case 1:
            return 'MMB';
        case 2:
            return 'RMB';
        default:
            return `Mouse${button}`;
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
