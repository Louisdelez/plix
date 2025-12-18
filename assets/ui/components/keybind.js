/**
 * Keybind Component (Feature 031)
 *
 * Editable keybind button with conflict detection.
 */

/**
 * Keybind editor component
 */
export class KeybindEditor {
    /**
     * @param {HTMLElement} container - Container element
     * @param {object} options - Editor options
     * @param {object} options.keybinds - Current keybind map
     * @param {Function} options.onChange - Change callback (action, key)
     */
    constructor(container, options = {}) {
        this.container = container;
        this.options = options;

        /** @type {object} Current keybinds */
        this.keybinds = { ...options.keybinds } || {};

        /** @type {string|null} Currently editing action */
        this.editing = null;

        this._keyHandler = null;
        this._mouseHandler = null;
    }

    /**
     * Render the keybind editor
     * @param {Array<{key: string, label: string}>} actions - Action definitions
     */
    render(actions) {
        this.container.innerHTML = actions
            .map(
                ({ key, label }) => `
            <div class="settings-row">
                <span class="settings-label">${escapeHtml(label)}</span>
                <button class="keybind-btn ${this.editing === key ? 'editing' : ''}"
                    data-action="${key}" id="keybind-${key}">
                    ${escapeHtml(this.keybinds[key] || '?')}
                </button>
            </div>
        `
            )
            .join('');

        this.attachListeners();
    }

    /**
     * Attach event listeners
     */
    attachListeners() {
        // Button clicks
        this.container.querySelectorAll('.keybind-btn').forEach((btn) => {
            btn.addEventListener('click', () => {
                const action = btn.dataset.action;
                this.startEdit(action);
            });
        });

        // Global key capture
        this._keyHandler = (e) => this.handleKeyDown(e);
        window.addEventListener('keydown', this._keyHandler);

        // Global mouse capture
        this._mouseHandler = (e) => this.handleMouseDown(e);
        window.addEventListener('mousedown', this._mouseHandler);
    }

    /**
     * Cleanup event listeners
     */
    destroy() {
        if (this._keyHandler) {
            window.removeEventListener('keydown', this._keyHandler);
        }
        if (this._mouseHandler) {
            window.removeEventListener('mousedown', this._mouseHandler);
        }
    }

    /**
     * Start editing a keybind
     * @param {string} action - Action key
     */
    startEdit(action) {
        // Toggle if clicking same action
        if (this.editing === action) {
            this.cancelEdit();
            return;
        }

        // Cancel any previous edit
        this.cancelEdit();

        this.editing = action;
        const btn = document.getElementById(`keybind-${action}`);
        if (btn) {
            btn.classList.add('editing');
            btn.textContent = 'Press key...';
        }
    }

    /**
     * Cancel current edit
     */
    cancelEdit() {
        if (!this.editing) return;

        const btn = document.getElementById(`keybind-${this.editing}`);
        if (btn) {
            btn.classList.remove('editing');
            btn.textContent = this.keybinds[this.editing] || '?';
        }
        this.editing = null;
    }

    /**
     * Handle keydown during edit
     * @param {KeyboardEvent} e
     */
    handleKeyDown(e) {
        if (!this.editing) return;

        // ESC cancels edit
        if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            this.cancelEdit();
            return;
        }

        e.preventDefault();
        e.stopPropagation();

        const keyName = getKeyName(e);
        if (!keyName) return;

        this.assignKey(this.editing, keyName);
    }

    /**
     * Handle mousedown during edit
     * @param {MouseEvent} e
     */
    handleMouseDown(e) {
        if (!this.editing) return;

        // Only capture mouse buttons 0-2
        if (e.button > 2) return;

        // Ignore clicks on keybind buttons
        if (e.target.classList.contains('keybind-btn')) return;

        e.preventDefault();
        e.stopPropagation();

        const buttonName = getMouseButtonName(e.button);
        this.assignKey(this.editing, buttonName);
    }

    /**
     * Assign a key to an action
     * @param {string} action - Action key
     * @param {string} keyName - Key name
     */
    assignKey(action, keyName) {
        // Check for conflict
        const conflict = Object.entries(this.keybinds).find(
            ([a, k]) => a !== action && k === keyName
        );

        if (conflict) {
            // Swap keybinds
            const [conflictAction] = conflict;
            const oldKey = this.keybinds[action];
            this.keybinds[conflictAction] = oldKey;
            this.keybinds[action] = keyName;

            // Update conflict button
            const conflictBtn = document.getElementById(`keybind-${conflictAction}`);
            if (conflictBtn) {
                conflictBtn.textContent = oldKey;
            }
        } else {
            this.keybinds[action] = keyName;
        }

        // Update button
        const btn = document.getElementById(`keybind-${action}`);
        if (btn) {
            btn.classList.remove('editing');
            btn.textContent = keyName;
        }

        this.editing = null;

        // Notify change
        if (this.options.onChange) {
            this.options.onChange(action, keyName, this.keybinds);
        }
    }

    /**
     * Get current keybinds
     * @returns {object}
     */
    getKeybinds() {
        return { ...this.keybinds };
    }

    /**
     * Set keybinds
     * @param {object} keybinds
     */
    setKeybinds(keybinds) {
        this.keybinds = { ...keybinds };
    }
}

/**
 * Get key name from keyboard event
 * @param {KeyboardEvent} e
 * @returns {string|null}
 */
function getKeyName(e) {
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

    // Letters
    if (e.code.startsWith('Key')) {
        return e.code.slice(3);
    }

    // Digits
    if (e.code.startsWith('Digit')) {
        return e.code.slice(5);
    }

    // Function keys
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
    if (str === null || str === undefined) return '';
    const div = document.createElement('div');
    div.textContent = String(str);
    return div.innerHTML;
}
