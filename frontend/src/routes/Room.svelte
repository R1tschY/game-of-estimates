<script>
    import { connected, player_id, game, vote } from '../stores.js'
    import Banner from '../components/Banner.svelte'
    import CopyLink from '../components/CopyLink.svelte'
    import { decks, get_deck } from '../consts'

    export let id = null
    if (id !== null && id !== $game.id) {
        console.log('init join')
        game.join(id)
    }

    function mapVotes() {
        let new_votes = []
        let game_votes = $game.state.votes
        $game.players.forEach((player) => {
            let player_id = player.id
            if (game_votes.hasOwnProperty(player_id)) {
                new_votes.push({ id: player_id, vote: game_votes[player_id] })
            }
        })
        return new_votes
    }

    function setVote(value) {
        vote.update((v) => {
            return v !== value ? value : null
        })
    }

    function forceOpen() {
        if (!open) {
            game.force_open()
        }
    }

    function restart() {
        game.restart()
    }

    $: cards = $game.state ? get_deck($game.state.deck).cards : []
    $: votes = $game.state ? mapVotes() : []
    $: open = $game.state && $game.state.open
</script>

<style>
    .game-card {
        width: 2.5em;
        height: calc(2.5em * 4 / 3);
        padding: 0;
        font-size: 2em;
        text-align: center;
        display: inline-block;
        line-height: 3em;
        margin: 10px 10px;
        border: 1px solid #999;
        box-shadow: 0 0em 0.5em -0.125em rgba(10, 10, 10, 0.1);
        outline: 0;
        transition: margin 0.5s;
        transition: transform 5s;
        border-radius: 8px;
        transform: rotateY(0deg);
        transition: transform 1s;
    }

    .game-card.selectable {
        cursor: pointer;
    }

    .game-card.active {
        background-color: orange;
        color: white;
    }

    .game-card.selected {
        margin-top: -0.5rem;
    }

    .game-card-item {
        display: inline-block;
        vertical-align: top;
        white-space: nowrap;
        list-style: none;
    }

    .game-card-back {
        background-color: whitesmoke;
        color: white;
        padding: 2px;
        backface-visibility: hidden;
        transform: rotateY(180deg);
        position: absolute;
    }

    .game-card-front {
        background-color: white;
        padding: 2px;
        backface-visibility: hidden;
        transform: rotateY(0deg);
        position: absolute;
    }

    .game-card-normal {
        position: unset;
        background-color: white;
    }

    .backcover > .game-card-back {
        transform: rotateY(0deg);
    }

    .backcover > .game-card-front {
        transform: rotateY(180deg);
    }

    .hidden > .game-card {
        display: none;
    }

    .empty {
        border: 2px solid transparent;
        box-shadow: inset 0 0em 0.5em -0.125em rgba(10, 10, 10, 0.1);
        z-index: -10;
        position: relative;
    }
</style>

<div>
    <Banner />
    <div class="container">
        <CopyLink />
    </div>

    <section class="section">
        <div class="container">
            <h2 class="title is-4">Estimates</h2>
            <ul>
                {#each votes as player_vote (player_vote.id)}
                <li class="game-card-item">
                    <div class:backcover={!open} class:hidden={!player_vote.vote}>
                        <div class="game-card game-card-back">
                            ♠️
                        </div>
                        <div class="game-card game-card-front">
                            {player_vote.vote ? player_vote.vote : '\xA0'}
                        </div>
                    </div>
                    <div class="game-card empty"></div>
                </li>
                {/each}
            </ul>
            <div>
                <button class="button is-primary"on:click={restart}>Restart</button>
                <button
                    class="button is-primary"
                    on:click={forceOpen}>Open</button>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <h2 class="title is-4">Choose your estimate</h2>
            <ul>
                {#each cards as card}
                    <li class="game-card-item">
                        <button
                            class="game-card game-card-normal selectable"
                            on:click={setVote(card)}
                            class:active={$vote === card}
                            class:selected={$vote === card}>
                            {card}
                        </button>
                    </li>
                {/each}
            </ul>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <div>Connected: {$connected}</div>
            <div>Player ID: {$player_id}</div>
            <div>ID: {id}</div>
            <div>game ID: {$game.id}</div>
            <div>game State: {$game.status}</div>
            <div>game Error: {$game.last_error}</div>
            <div>game state: {JSON.stringify($game.state)}</div>
            <div>votes: {JSON.stringify(votes)}</div>
            <div>vote: {$vote}</div>
            <div>Open: {open}</div>
            <div>game players: {JSON.stringify($game.players)}</div>
        </div>
    </section>
</div>
