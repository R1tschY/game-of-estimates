<script lang="ts">
    import { gameState, name, observer, players } from '$/stores'

    import { get } from 'svelte/store'
    import DisconnectedMW from '$/components/organisms/DisconnectedMW.svelte'
    import SingleTextInput from '$/components/atoms/SingleTextInput.svelte'
    import Switch from '$/components/atoms/Switch.svelte'
    import PlayerEstimate from '$/components/atoms/PlayerEstimate.svelte'
    import EstimatesControl from '$/components/molecules/EstimatesControl.svelte'
    import { client, wsService } from '$/client'
    import { m } from '$lib/paraglide/messages.js'
    import { onMount } from 'svelte'

    let { id } = $props()

    let newName: string = $state($name ?? '')

    const open: boolean = $derived($gameState?.open ?? false)

    onMount(() => {
        console.log('onMount')
        wsService.connect()
    })

    // TODO: disconnect on unmount
    client.welcome.connect(() => {
        const state = get(client.state)
        if (state !== 'joining' && state !== 'joined') {
            console.log('init join', id)
            client.joinRoom(id)
        }
    })

    function forceOpen() {
        if (!open) {
            client.forceOpen()
        }
    }

    function restart() {
        client.restart()
    }

    function changeName() {
        $name = newName ?? ''
    }
</script>

<section class="section">
    <div class="container box">
        <div class="columns">
            <!-- Name -->
            <div class="column player-name-control">
                <SingleTextInput
                    id="player-name"
                    action="✓"
                    placeholder={m.playerNamePlaceholder()}
                    bind:value={newName}
                    on:submit={changeName}
                />
            </div>

            <div class="column"></div>

            <!-- Voter -->
            <div class="column is-narrow">
                <Switch
                    id="player-is-voter"
                    class="player-name-control"
                    bind:value={$observer}
                    label={m.observer()}
                />
            </div>
        </div>
    </div>
</section>

<section class="section">
    <div class="container box">
        <h2 class="title is-4">{m.estimates()}</h2>
        <div class="buttons">
            <button class="button is-primary is-light" onclick={restart}
                >{m.restart()}</button
            >
            <button
                class="button is-primary is-light"
                disabled={open}
                onclick={forceOpen}>{m.open()}</button
            >
        </div>
        <ul class="game-board">
            {#each $players as player (player.id)}
                {#if player.voter}
                    <PlayerEstimate {player} {open} />
                {/if}
            {/each}
        </ul>
    </div>
</section>

{#if !$observer}
    <section class="section">
        <div class="container">
            <div class="columns is-centered">
                <div class="column is-narrow">
                    {m.chooseYourEstimate()}
                </div>
            </div>
            <EstimatesControl />
        </div>
    </section>
{/if}

<DisconnectedMW />
