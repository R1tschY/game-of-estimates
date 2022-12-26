<script lang="ts">
    import {vote, gameState} from '../stores'
    import {get_deck as getDeck} from '../deck'

    $: // noinspection TypeScriptValidateTypes
        open = $gameState ? $gameState.open : false
    $: cards = $gameState ? getDeck($gameState.deck).cards : []

    let touch_hover: string | null = null;
    let handNode

    function setVote(value: string | null) {
        if (!open) {
            vote.update((v) => {
                return v !== value ? value : null
            })
        }
    }

    function touchHover(evt: TouchEvent) {
        if (evt.touches.length === 1) {
            const touch = evt.touches.item(0)
            const cardsValue = cards;
            const rect = handNode.getBoundingClientRect()

            let fpos = (touch.clientX - rect.left) * cardsValue.length / (rect.right - rect.left);
            const pos = Math.min(cardsValue.length - 1, Math.max(0, Math.round(fpos)))
            touch_hover = cardsValue[pos]
            evt.preventDefault()
        }
    }

    function touchHoverEnd() {
        setVote(touch_hover)
        touch_hover = null
    }

    function touchHoverCancel() {
        setVote(null)
        touch_hover = null
    }
</script>

<ul class="hand" bind:this={handNode} on:touchstart={touchHover} on:touchmove={touchHover} on:touchend={touchHoverEnd}
    on:touchcancel={touchHoverCancel}>
    {#each cards as card}
        <li class="hand-entry">
            <div class="game-card-item hand-entry-inner">
                <button
                        class="game-card game-card-normal"
                        type="button"
                        on:click={() => setVote(card)}
                        on:touchmove={touchHover}
                        class:selected={$vote === card}
                        class:touch-hover={touch_hover === card}
                        class:selectable={!open}
                >
                    <span class="game-card-inner">{card}</span>
                </button>
            </div>
        </li>
    {/each}
</ul>
