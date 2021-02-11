<style lang="scss">
    $card-width: 63mm * 0.25;
    $card-height: 88mm * 0.25;

    $tt-gray:#63666a;
    $tt-orange: #ed8b00;
    $tt-petrol: #208CA3;

    .card-inner {
        border: 1px solid #eee;
        border-radius: 7px 0 7px 0;
        margin: 3px;
        line-height: calc(#{$card-height} - 10px);
        vertical-align: middle;
    }

    .game-card {
        width: $card-width;
        height: $card-height;
        padding: 0;
        font-size: 1.7em;
        text-align: center;
        display: inline-block;
        // line-height: 3em;
        margin: 0 7px;
        border: 1px solid $tt-gray;
        box-shadow: 0 0em 0.5em -0.125em rgba(10, 10, 10, 0.1);
        outline: 0;
        transition: transform 1s, margin-top 0.2s;
        border-radius: 10px 0 10px 0;
        transform: rotateY(0deg);
    }

    .game-card.selectable {
        cursor: pointer;
    }  

    .game-card.selected {
        background-color: #{adjust-color($tt-orange, $lightness: 40%)};
        //color: white;
        margin-top: -0.5rem;
    }

    .game-card-item {
        display: inline-block;
        vertical-align: top;
        white-space: nowrap;
        list-style: none;
    }

    .game-card-back {
        background-color: $tt-orange;
        color: black;
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

    .card-row {
        height: $card-height;
        margin: 10px 0;
    }
</style>

<script lang="ts">
    import { connected, player_id, room, vote, voter } from '../stores'
    import Banner from '../components/Banner.svelte'
    import CopyLink from '../components/CopyLink.svelte'
    import { get_deck } from '../deck'

    export let id: string | null = null
    if (id !== null && id !== $room.id) {
        console.log('init join')
        room.join(id)
    }

    function mapVotes() {
        let new_votes = []
        let game_votes = $room.state.votes
        $room.players.forEach((player) => {
            let player_id = player.id
            if (game_votes.hasOwnProperty(player_id)) {
                new_votes.push({ id: player_id, vote: game_votes[player_id] })
            }
        })
        return new_votes
    }

    function setVote(value: string | null) {
        vote.update((v) => {
            return v !== value ? value : null
        })
    }

    function updateVoter(value: boolean) {
        voter.update((v) => {
            return v !== value ? value : null
        })
    }

    function forceOpen() {
        if (!open) {
            room.force_open()
        }
    }

    function restart() {
        room.restart()
    }

    $: cards = $room.state ? get_deck($room.state.deck).cards : []
    $: votes = $room.state ? mapVotes() : []
    $: open = $room.state && $room.state.open
</script>

<div>
    <Banner />
    <div class="container">
        <div class="field">
            <CopyLink />
        </div>

        <div class="field">
            <!-- User -->
            <input id="voterField" type="checkbox" class="switch" bind:checked={$voter}>
            <label for="voterField">Voter</label>
        </div>
    </div>

    <section class="section">
        <div class="container">
            <h2 class="title is-4">Estimates</h2>
            <ul class="card-row">
                {#each votes as player_vote (player_vote.id)}
                <li class="game-card-item">
                    <div class:backcover={!open} class:hidden={!player_vote.vote}>
                        <div class="game-card game-card-back">
                            <div class="card-inner">♠️</div>
                        </div>
                        <div class="game-card game-card-front">
                            <div class="card-inner">{player_vote.vote ? player_vote.vote : '\xA0'}</div>
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
            <ul class="card-row">
                {#each cards as card}
                    <li class="game-card-item">
                        <button
                            class="game-card game-card-normal selectable"
                            on:click={() => setVote(card)}
                            class:selected={$vote === card}>
                            <div class="card-inner">{card}</div>
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
            <div>game ID: {$room.id}</div>
            <div>game State: {$room.status}</div>
            <div>game Error: {$room.last_error}</div>
            <div>game state: {JSON.stringify($room.state)}</div>
            <div>votes: {JSON.stringify(votes)}</div>
            <div>vote: {$vote}</div>
            <div>voter: {voter}</div>
            <div>Open: {open}</div>
            <div>game players: {JSON.stringify($room.players)}</div>
        </div>
    </section>
</div>
