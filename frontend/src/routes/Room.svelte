<script>
    import { connected, player_id, game, vote } from '../stores.js'
    import Banner from '../components/Banner.svelte'

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

    export let id = null
    if (id !== null && id !== $game.id) {
        console.log('init join')
        game.join(id)
    }

    $: cards = $game.state ? $game.state.cards : []
    $: votes = $game.state ? mapVotes() : []
</script>

<style>
    .game-card {
        width: 2.5em;
        height: calc(2.5em * 4 / 3);
        font-size: 2em;
        text-align: center;
        padding: 2px;
        display: inline-block;
        line-height: 3em;
        margin: 10px 10px;
    }

    .game-card.active {
        background-color: bisque;
    }
</style>

<div>
    <Banner />
    <section class="section">
        <div class="container">
            <h2 class="title is-4">Your estimate</h2>
            {#each cards as card}
                <div
                    class="box game-card"
                    on:click={setVote(card)}
                    class:active={$vote === card}>
                    {card}
                </div>
            {/each}
        </div>
    </section>
    <section class="section">
        <div class="container">
            <h2 class="title is-4">Estimates</h2>
            {#each votes as player_vote (player_vote.id)}
                <div class="box game-card">{player_vote.vote}</div>
            {/each}
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
            <div>game players: {JSON.stringify($game.players)}</div>
        </div>
    </section>
</div>
