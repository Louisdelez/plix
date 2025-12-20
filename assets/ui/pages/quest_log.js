/**
 * Quest Log Page (Feature 043)
 *
 * Displays active and completed quests with progress tracking.
 */

export class QuestLogPage {
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
        this.activeQuests = [];
        this.completedQuestIds = [];
        this.selectedQuestId = null;
    }

    /**
     * Render the page HTML
     * @returns {string}
     */
    render() {
        return `
            <div class="page quest-log-page">
                <div class="page-header">
                    <button class="btn btn-back" id="btn-back">&larr; Back</button>
                    <h1 class="page-title">Quest Log</h1>
                </div>

                <div class="quest-log-container">
                    <div class="quest-list-panel">
                        <div class="quest-tabs">
                            <button class="tab-btn active" data-tab="active">Active</button>
                            <button class="tab-btn" data-tab="completed">Completed</button>
                        </div>
                        <div id="quest-list" class="quest-list">
                            <p class="quest-list-empty">Loading quests...</p>
                        </div>
                    </div>

                    <div class="quest-detail-panel" id="quest-detail">
                        <p class="quest-detail-empty">Select a quest to view details</p>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * Initialize event listeners
     */
    async init() {
        // Back button
        document.getElementById('btn-back')?.addEventListener('click', () => {
            this.navigate('/main');
        });

        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });

        // Listen for quest updates from server
        this.bridge.on('QuestSync', (data) => this.onQuestSync(data));
        this.bridge.on('QuestUpdate', (data) => this.onQuestUpdate(data));

        // Request initial quest data
        await this.requestQuestSync();
    }

    /**
     * Request quest sync from server
     */
    async requestQuestSync() {
        try {
            await this.bridge.send('QuestSyncRequest', {});
        } catch (e) {
            console.error('[QuestLog] Failed to request quest sync:', e);
        }
    }

    /**
     * Handle quest sync from server
     * @param {object} data
     */
    onQuestSync(data) {
        this.activeQuests = data.active_quests || [];
        this.completedQuestIds = data.completed_quest_ids || [];
        this.renderQuestList('active');
    }

    /**
     * Handle quest update from server
     * @param {object} data
     */
    onQuestUpdate(data) {
        const { quest_id, update_type } = data;

        switch (update_type) {
            case 'Started':
            case 'StepProgress':
            case 'StepAdvanced':
                // Re-sync to get latest data
                this.requestQuestSync();
                break;
            case 'Completed':
            case 'Abandoned':
                // Re-sync
                this.requestQuestSync();
                break;
        }
    }

    /**
     * Switch between active/completed tabs
     * @param {string} tab
     */
    switchTab(tab) {
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.tab === tab);
        });
        this.renderQuestList(tab);
    }

    /**
     * Render the quest list
     * @param {string} tab - 'active' or 'completed'
     */
    renderQuestList(tab) {
        const container = document.getElementById('quest-list');
        if (!container) return;

        if (tab === 'active') {
            if (this.activeQuests.length === 0) {
                container.innerHTML = '<p class="quest-list-empty">No active quests</p>';
                return;
            }

            container.innerHTML = this.activeQuests.map(quest => `
                <div class="quest-list-item ${this.selectedQuestId === quest.quest_id ? 'selected' : ''}"
                     data-quest-id="${escapeHtml(quest.quest_id)}">
                    <div class="quest-list-item-header">
                        <span class="quest-title">${escapeHtml(quest.title)}</span>
                        ${quest.pinned ? '<span class="quest-pinned-badge">Pinned</span>' : ''}
                    </div>
                    <div class="quest-progress-bar">
                        <div class="quest-progress-fill" style="width: ${this.calculateProgress(quest)}%"></div>
                    </div>
                    <p class="quest-step-preview">${escapeHtml(this.getCurrentStepText(quest))}</p>
                </div>
            `).join('');
        } else {
            if (this.completedQuestIds.length === 0) {
                container.innerHTML = '<p class="quest-list-empty">No completed quests</p>';
                return;
            }

            container.innerHTML = this.completedQuestIds.map(id => `
                <div class="quest-list-item completed" data-quest-id="${escapeHtml(id)}">
                    <span class="quest-title">${escapeHtml(id)}</span>
                    <span class="quest-completed-badge">Completed</span>
                </div>
            `).join('');
        }

        // Add click handlers
        container.querySelectorAll('.quest-list-item').forEach(item => {
            item.addEventListener('click', () => {
                this.selectQuest(item.dataset.questId);
            });
        });
    }

    /**
     * Calculate quest progress percentage
     * @param {object} quest
     * @returns {number}
     */
    calculateProgress(quest) {
        if (!quest.steps || quest.steps.length === 0) return 100;

        const completedSteps = quest.steps.filter(s => s.progress?.completed).length;
        const currentStepProgress = quest.steps[quest.current_step]?.progress;

        let progress = (completedSteps / quest.steps.length) * 100;

        if (currentStepProgress && !currentStepProgress.completed) {
            const stepProgress = (currentStepProgress.current / currentStepProgress.target) * (100 / quest.steps.length);
            progress += stepProgress;
        }

        return Math.min(100, Math.round(progress));
    }

    /**
     * Get current step description
     * @param {object} quest
     * @returns {string}
     */
    getCurrentStepText(quest) {
        const step = quest.steps?.[quest.current_step];
        if (!step) return 'Quest complete';

        const progress = step.progress;
        if (progress && progress.target > 1) {
            return `${step.description} (${progress.current}/${progress.target})`;
        }
        return step.description;
    }

    /**
     * Select a quest and show details
     * @param {string} questId
     */
    selectQuest(questId) {
        this.selectedQuestId = questId;

        // Update selection in list
        document.querySelectorAll('.quest-list-item').forEach(item => {
            item.classList.toggle('selected', item.dataset.questId === questId);
        });

        // Find quest data
        const quest = this.activeQuests.find(q => q.quest_id === questId);
        this.renderQuestDetail(quest);
    }

    /**
     * Render quest detail panel
     * @param {object|undefined} quest
     */
    renderQuestDetail(quest) {
        const container = document.getElementById('quest-detail');
        if (!container) return;

        if (!quest) {
            container.innerHTML = '<p class="quest-detail-empty">Select a quest to view details</p>';
            return;
        }

        container.innerHTML = `
            <div class="quest-detail-header">
                <h2>${escapeHtml(quest.title)}</h2>
                <div class="quest-detail-actions">
                    <button class="btn btn-small ${quest.pinned ? 'btn-primary' : ''}" id="btn-pin">
                        ${quest.pinned ? 'Unpin' : 'Pin to HUD'}
                    </button>
                    <button class="btn btn-small btn-danger" id="btn-abandon">Abandon</button>
                </div>
            </div>

            <p class="quest-detail-description">${escapeHtml(quest.description)}</p>

            <h3>Objectives</h3>
            <ul class="quest-steps-list">
                ${quest.steps.map((step, i) => `
                    <li class="quest-step ${step.progress?.completed ? 'completed' : ''} ${i === quest.current_step ? 'current' : ''}">
                        <span class="step-checkbox">${step.progress?.completed ? '&#10003;' : (i === quest.current_step ? '&bull;' : '&circ;')}</span>
                        <span class="step-text">${escapeHtml(step.description)}</span>
                        ${step.progress && step.progress.target > 1 ? `
                            <span class="step-progress">(${step.progress.current}/${step.progress.target})</span>
                        ` : ''}
                    </li>
                `).join('')}
            </ul>
        `;

        // Pin button
        document.getElementById('btn-pin')?.addEventListener('click', async () => {
            try {
                await this.bridge.send('QuestPin', {
                    quest_id: quest.pinned ? null : quest.quest_id
                });
                await this.requestQuestSync();
            } catch (e) {
                console.error('[QuestLog] Failed to pin quest:', e);
            }
        });

        // Abandon button
        document.getElementById('btn-abandon')?.addEventListener('click', async () => {
            if (!confirm(`Abandon quest "${quest.title}"? Progress will be lost.`)) {
                return;
            }

            try {
                await this.bridge.send('QuestAbandon', { quest_id: quest.quest_id });
                this.selectedQuestId = null;
                await this.requestQuestSync();
            } catch (e) {
                console.error('[QuestLog] Failed to abandon quest:', e);
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
    if (!str) return '';
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}
