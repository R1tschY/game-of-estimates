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

<style lang="sass">
    .card-row
        display: flex
        justify-content: space-between

    .card-row-entry
        position: relative

    .card-row-entry-inner
        position: absolute
        left: -8mm // TODO
</style>

<ul class="card-row">
    {#each cards as card}
        <li class="card-row-entry">
            <div class="game-card-item card-row-entry-inner">
                <button
                    class="game-card game-card-normal"
                    type="button"
                    on:click={() => setVote(card)}
                    class:selected={$vote === card}
                    class:selectable={!open}
                >
                    <div class="game-card-inner">{card}</div>
                </button>
            </div>
        </li>
    {/each}
</ul>
