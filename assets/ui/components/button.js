/**
 * Button Component (Feature 031)
 *
 * Debounced button with loading state support.
 */

// Default debounce delay in ms
const DEFAULT_DEBOUNCE = 500;

/**
 * Create a debounced button handler
 * @param {HTMLElement} btn - Button element
 * @param {Function} handler - Click handler (can be async)
 * @param {number} [debounceMs=500] - Debounce delay in ms
 */
export function createDebouncedButton(btn, handler, debounceMs = DEFAULT_DEBOUNCE) {
    let lastClick = 0;
    let isLoading = false;

    btn.addEventListener('click', async (e) => {
        // Debounce check
        const now = Date.now();
        if (now - lastClick < debounceMs) {
            e.preventDefault();
            return;
        }

        // Already loading check
        if (isLoading) {
            e.preventDefault();
            return;
        }

        lastClick = now;

        // Set loading state
        setButtonLoading(btn, true);
        isLoading = true;

        try {
            await handler(e);
        } finally {
            setButtonLoading(btn, false);
            isLoading = false;
        }
    });
}

/**
 * Set button loading state
 * @param {HTMLElement} btn - Button element
 * @param {boolean} loading - Loading state
 */
export function setButtonLoading(btn, loading) {
    if (loading) {
        btn.disabled = true;
        btn.classList.add('btn-loading');
        btn.dataset.originalText = btn.textContent;
    } else {
        btn.disabled = false;
        btn.classList.remove('btn-loading');
        if (btn.dataset.originalText) {
            btn.textContent = btn.dataset.originalText;
            delete btn.dataset.originalText;
        }
    }
}

/**
 * Disable button
 * @param {HTMLElement} btn - Button element
 * @param {boolean} disabled - Disabled state
 */
export function setButtonDisabled(btn, disabled) {
    btn.disabled = disabled;
}

/**
 * Create a button element with debounce
 * @param {object} options - Button options
 * @param {string} options.label - Button label
 * @param {string} [options.id] - Button ID
 * @param {string} [options.className] - Additional class names
 * @param {boolean} [options.primary=false] - Primary style
 * @param {boolean} [options.disabled=false] - Disabled state
 * @param {Function} options.onClick - Click handler
 * @param {number} [options.debounce=500] - Debounce delay
 * @returns {HTMLButtonElement}
 */
export function createButton(options) {
    const {
        label,
        id,
        className = '',
        primary = false,
        disabled = false,
        onClick,
        debounce = DEFAULT_DEBOUNCE,
    } = options;

    const btn = document.createElement('button');
    btn.className = `btn ${primary ? 'btn-primary' : ''} ${className}`.trim();
    btn.textContent = label;
    btn.disabled = disabled;

    if (id) {
        btn.id = id;
    }

    if (onClick) {
        createDebouncedButton(btn, onClick, debounce);
    }

    return btn;
}
