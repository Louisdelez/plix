/**
 * Quest Tracker HUD Overlay (Feature 043)
 *
 * Displays the currently pinned quest in the game HUD.
 */

export class QuestTracker {
    /**
     * @param {object} bridge - Bridge interface for server communication
     */
    constructor(bridge) {
        this.bridge = bridge;
        this.currentQuest = null;
        this.container = null;
        this.visible = false;
    }

    /**
     * Initialize the quest tracker
     */
    init() {
        // Create container element
        this.container = document.createElement('div');
        this.container.id = 'quest-tracker';
        this.container.className = 'quest-tracker hidden';
        this.container.innerHTML = this.renderEmpty();
        document.body.appendChild(this.container);

        // Listen for quest tracker updates
        this.bridge.on('QuestTrackerUpdate', (data) => this.onTrackerUpdate(data));
        this.bridge.on('QuestUpdate', (data) => this.onQuestUpdate(data));
    }

    /**
     * Render empty state
     * @returns {string}
     */
    renderEmpty() {
        return `
            <div class="quest-tracker-content">
                <p class="quest-tracker-empty">No quest pinned</p>
            </div>
        `;
    }

    /**
     * Handle quest tracker update from server
     * @param {object} data
     */
    onTrackerUpdate(data) {
        this.currentQuest = data;
        this.render();
        this.show();
    }

    /**
     * Handle general quest update
     * @param {object} data
     */
    onQuestUpdate(data) {
        if (!this.currentQuest) return;

        // If this update is for our tracked quest
        if (data.quest_id === this.currentQuest.quest_id) {
            switch (data.update_type) {
                case 'StepProgress':
                    if (data.progress !== undefined && data.target !== undefined) {
                        this.currentQuest.progress = data.progress;
                        this.currentQuest.target = data.target;
                        this.render();
                    }
                    break;
                case 'StepAdvanced':
                    // Wait for QuestTrackerUpdate with new step info
                    break;
                case 'Completed':
                case 'Abandoned':
                    this.currentQuest = null;
                    this.hide();
                    break;
            }
        }
    }

    /**
     * Render the quest tracker
     */
    render() {
        if (!this.container || !this.currentQuest) {
            if (this.container) {
                this.container.innerHTML = this.renderEmpty();
            }
            return;
        }

        const quest = this.currentQuest;
        const hasProgress = quest.target > 1;
        const progressPercent = hasProgress ? Math.round((quest.progress / quest.target) * 100) : 0;

        this.container.innerHTML = `
            <div class="quest-tracker-content">
                <div class="quest-tracker-header">
                    <span class="quest-tracker-title">${escapeHtml(quest.title)}</span>
                </div>
                <div class="quest-tracker-step">
                    <p class="quest-tracker-step-text">${escapeHtml(quest.step_description)}</p>
                    ${hasProgress ? `
                        <div class="quest-tracker-progress">
                            <div class="quest-tracker-progress-bar">
                                <div class="quest-tracker-progress-fill" style="width: ${progressPercent}%"></div>
                            </div>
                            <span class="quest-tracker-progress-text">${quest.progress}/${quest.target}</span>
                        </div>
                    ` : ''}
                </div>
            </div>
        `;
    }

    /**
     * Show the tracker
     */
    show() {
        if (this.container) {
            this.container.classList.remove('hidden');
            this.visible = true;
        }
    }

    /**
     * Hide the tracker
     */
    hide() {
        if (this.container) {
            this.container.classList.add('hidden');
            this.visible = false;
        }
    }

    /**
     * Toggle visibility
     */
    toggle() {
        if (this.visible) {
            this.hide();
        } else {
            this.show();
        }
    }

    /**
     * Clean up
     */
    destroy() {
        if (this.container) {
            this.container.remove();
            this.container = null;
        }
    }
}

/**
 * Escape HTML special characters
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
    if (!str) return '';
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}
