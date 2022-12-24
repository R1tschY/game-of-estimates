<script lang="ts">
    import { vote, gameState } from '../stores'
    import { get_deck as getDeck } from '../deck'

    $: open = $gameState && $gameState.open
    $: cards = $gameState ? getDeck($gameState.deck).cards : []

    function setVote(value: string | null) {
        if (!open) {
            vote.update((v) => {
                return v !== value ? value : null
            })
        }
    }
</script>

<ul class="hand">
    {#each cards as card}
        <li class="hand-entry">
            <div class="game-card-item hand-entry-inner">
                <button
                    class="game-card game-card-normal"
                    type="button"
                    on:click={() => setVote(card)}
                    class:selected={$vote === card}
                    class:selectable={!open}
                >
                    <span class="game-card-inner">{card}</span>
                </button>
            </div>
        </li>
    {/each}
</ul>
