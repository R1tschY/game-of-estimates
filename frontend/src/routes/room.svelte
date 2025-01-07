<script lang="ts"
        ✂prettier:content✂="CiAgICBpbXBvcnQgewogICAgICAgIGRlYnVnLAogICAgICAgIGdhbWVTdGF0ZSwKICAgICAgICBuYW1lIGFzIG5hbWVTdG9yZSwKICAgICAgICBwbGF5ZXJJZCwKICAgICAgICBwbGF5ZXJzLAogICAgICAgIHZvdGUsCiAgICAgICAgb2JzZXJ2ZXIsCiAgICB9IGZyb20gJy4uL3N0b3JlcycKICAgIGltcG9ydCBIZWFkZXIgZnJvbSAnLi4vY29tcG9uZW50cy9IZWFkZXIuc3ZlbHRlJwoKICAgIGltcG9ydCB7IGNsaWVudCwgcGxheWVyU3RhdGUgfSBmcm9tICcuLi9jbGllbnQnCiAgICBpbXBvcnQgeyBnZXQgfSBmcm9tICdzdmVsdGUvc3RvcmUnCiAgICBpbXBvcnQgRGlzY29ubmVjdGVkTVcgZnJvbSAnLi4vY29tcG9uZW50cy9EaXNjb25uZWN0ZWRNVy5zdmVsdGUnCiAgICBpbXBvcnQgU2luZ2xlVGV4dElucHV0IGZyb20gJy4uL2NvbXBvbmVudHMvU2luZ2xlVGV4dElucHV0LnN2ZWx0ZScKICAgIGltcG9ydCBTd2l0Y2ggZnJvbSAnLi4vY29tcG9uZW50cy9Td2l0Y2guc3ZlbHRlJwogICAgaW1wb3J0IEZvb3RlciBmcm9tICcuLi9jb21wb25lbnRzL0Zvb3Rlci5zdmVsdGUnCiAgICBpbXBvcnQgUGxheWVyRXN0aW1hdGUgZnJvbSAnLi4vY29tcG9uZW50cy9QbGF5ZXJFc3RpbWF0ZS5zdmVsdGUnCiAgICBpbXBvcnQgRXN0aW1hdGVzQ29udHJvbCBmcm9tICcuLi9jb21wb25lbnRzL0VzdGltYXRlc0NvbnRyb2wuc3ZlbHRlJwogICAgaW1wb3J0IHsgZ2V0VGV4dCB9IGZyb20gJy4uL2kxOG4nCgogICAgZXhwb3J0IGxldCBpZDogc3RyaW5nIHwgbnVsbCA9IG51bGwKCiAgICBsZXQgbmFtZTogc3RyaW5nID0gZ2V0KG5hbWVTdG9yZSkgPz8gJycKCiAgICBsZXQgb3BlbjogYm9vbGVhbgogICAgJDogb3BlbiA9ICRnYW1lU3RhdGU/Lm9wZW4gPz8gZmFsc2UKCiAgICAvLyBUT0RPOiBkaXNjb25uZWN0IG9uIHVubW91bnQKICAgIGNsaWVudC53ZWxjb21lLmNvbm5lY3QoKCkgPT4gewogICAgICAgIGNvbnN0IHN0YXRlID0gZ2V0KGNsaWVudC5zdGF0ZSkKICAgICAgICBpZiAoaWQgIT09IG51bGwgJiYgc3RhdGUgIT09ICdqb2luaW5nJyAmJiBzdGF0ZSAhPT0gJ2pvaW5lZCcpIHsKICAgICAgICAgICAgY29uc29sZS5sb2coJ2luaXQgam9pbicsIGlkKQogICAgICAgICAgICBjbGllbnQuam9pblJvb20oaWQpCiAgICAgICAgfQogICAgfSkKCiAgICBmdW5jdGlvbiBmb3JjZU9wZW4oKSB7CiAgICAgICAgaWYgKCFvcGVuKSB7CiAgICAgICAgICAgIGNsaWVudC5mb3JjZU9wZW4oKQogICAgICAgIH0KICAgIH0KCiAgICBmdW5jdGlvbiByZXN0YXJ0KCkgewogICAgICAgIGNsaWVudC5yZXN0YXJ0KCkKICAgIH0KCiAgICBmdW5jdGlvbiBjaGFuZ2VOYW1lKCkgewogICAgICAgIG5hbWVTdG9yZS5zZXQobmFtZSA/IG5hbWUgOiBudWxsKQogICAgfQo=">{
}
</script>

<div id="room-container">
    <Header/>

    <section class="section">
        <div class="container player-section">
            <div class="columns">
                <!-- Name -->
                <div class="column player-name-control">
                    <SingleTextInput
                            id="player-name"
                            action="✓"
                            placeholder={getText('playerNamePlaceholder')}
                            bind:value={name}
                            on:submit={changeName}
                    />
                </div>

                <div class="column"/>

                <!-- Voter -->
                <div class="column is-narrow">
                    <Switch
                            id="player-is-voter"
                            class="player-name-control"
                            bind:value={$observer}
                            label={getText('observer')}
                    />
                </div>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container estimates-section">
            <h2 class="title is-4">{getText('estimates')}</h2>
            <div class="buttons">
                <button class="button is-primary is-light" on:click={restart}
                >{getText('restart')}</button
                >
                <button
                        class="button is-primary is-light"
                        disabled={open}
                        on:click={forceOpen}>{getText('open')}</button
                >
            </div>
            <ul class="game-board">
                {#each $players as player (player.id)}
                    {#if player.voter}
                        <PlayerEstimate {player} {open}/>
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
                        {getText('chooseYourEstimate')}
                    </div>
                </div>
                <EstimatesControl/>
            </div>
        </section>
    {/if}

    {#if $debug}
        <section class="section">
            <div class="container">
                <div>player state: {$playerState}</div>
                <div>Player ID: {$playerId}</div>
                <div>ID: {id}</div>
                <div>vote: {$vote}</div>
                <div>voter: {!$observer}</div>
                <div>Open: {open}</div>
                <div>players: {JSON.stringify($players)}</div>
            </div>
        </section>
    {/if}

    <DisconnectedMW/>
    <Footer/>
</div>
