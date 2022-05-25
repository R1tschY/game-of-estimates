<script lang="ts">
    import type { PlayerExtInfo } from '../stores'

    export let player: PlayerExtInfo
    export let open: boolean
</script>

<li class="game-card-box">
    <div class="game-card-item">
        <div class:backcover={!open} class:hidden={!player.vote}>
            <div class="game-card game-card-back">
                <div class="game-card-inner">
                    <img class="game-card-symbol" src="/clubs.svg" alt="&spades;" />
                </div>
            </div>
            <div class="game-card game-card-front">
                <div class="game-card-inner">
                    {player.vote ? player.vote : '\xA0'}
                </div>
            </div>
        </div>
        <div class="game-card placeholder" />
    </div>
    <div class="game-card-name">
        {player.name || ""}
    </div>
</li>
