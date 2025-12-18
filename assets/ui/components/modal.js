/**
 * Modal Component (Feature 031)
 *
 * Reusable modal dialog for errors, loading, and confirmations.
 */

/**
 * Show a modal dialog
 * @param {HTMLElement} container - Container element for the modal
 * @param {object} options - Modal options
 * @param {string} options.title - Modal title
 * @param {string} options.content - Modal content (can be HTML)
 * @param {Array<{label: string, primary?: boolean, action: Function}>} options.actions - Button actions
 * @param {boolean} [options.isRaw=false] - If true, content is not escaped
 */
export function showModal(container, options) {
    const { title, content, actions = [], isRaw = false } = options;

    container.innerHTML = `
        <div class="modal-overlay" data-modal>
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

    // Close on overlay click (optional)
    const overlay = container.querySelector('.modal-overlay');
    overlay?.addEventListener('click', (e) => {
        if (e.target === overlay) {
            // Optional: close on backdrop click
            // hideModal(container);
        }
    });
}

/**
 * Hide the modal
 * @param {HTMLElement} container - Container element
 */
export function hideModal(container) {
    container.innerHTML = '';
}

/**
 * Show a loading modal
 * @param {HTMLElement} container - Container element
 * @param {string} title - Modal title
 * @param {string} message - Loading message
 * @param {Function} [onCancel] - Cancel callback
 */
export function showLoadingModal(container, title, message, onCancel) {
    const actions = onCancel
        ? [{ label: 'Cancel', action: onCancel }]
        : [];

    showModal(container, {
        title,
        content: `
            <div class="loading">
                <div class="loading-spinner"></div>
                <div class="loading-text">${escapeHtml(message)}</div>
            </div>
        `,
        actions,
        isRaw: true,
    });
}

/**
 * Show an error modal
 * @param {HTMLElement} container - Container element
 * @param {string} title - Modal title
 * @param {string} message - Error message
 * @param {Function} onClose - Close callback
 */
export function showErrorModal(container, title, message, onClose) {
    showModal(container, {
        title,
        content: `<div class="error-message">${escapeHtml(message)}</div>`,
        actions: [{ label: 'OK', primary: true, action: onClose }],
        isRaw: true,
    });
}

/**
 * Show a success modal
 * @param {HTMLElement} container - Container element
 * @param {string} title - Modal title
 * @param {string} message - Success message
 * @param {Function} onClose - Close callback
 */
export function showSuccessModal(container, title, message, onClose) {
    showModal(container, {
        title,
        content: `<div class="success-message">${escapeHtml(message)}</div>`,
        actions: [{ label: 'OK', primary: true, action: onClose }],
        isRaw: true,
    });
}

/**
 * Show a confirmation modal
 * @param {HTMLElement} container - Container element
 * @param {string} title - Modal title
 * @param {string} message - Confirmation message
 * @param {Function} onConfirm - Confirm callback
 * @param {Function} onCancel - Cancel callback
 */
export function showConfirmModal(container, title, message, onConfirm, onCancel) {
    showModal(container, {
        title,
        content: message,
        actions: [
            { label: 'Cancel', action: onCancel },
            { label: 'Confirm', primary: true, action: onConfirm },
        ],
    });
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
