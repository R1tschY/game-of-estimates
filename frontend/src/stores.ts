import { derived, get, writable } from 'svelte/store'
import type { Readable, Writable } from 'svelte/store'
import { navigate } from 'svelte-routing'
import { client, wsService } from './client'
import type { GameState, PlayerState } from './client'
import type { Option } from './basetypes'
import { writableLocalStorage as writableLocalStorage } from './store-utils'

// state

export const connected: Readable<boolean> = wsService.connected_store
export const connecting: Readable<boolean> = wsService.connecting_store

export interface PlayerSettings {
    name: Option<string>
    voter: boolean
    debug: boolean
}

export const ownPlayerState: Writable<PlayerSettings> = writableLocalStorage(
    'goe-player-settings',
    {
        name: null,
        voter: true,
        debug: false,
    },
)
ownPlayerState.subscribe((value) => {
    client.updatePlayer(value.voter, value.name)
})

export const debug: Readable<boolean> = derived(
    ownPlayerState,
    (value) => value.debug,
)

export const voter: Writable<boolean> = writable(get(ownPlayerState).voter)
voter.subscribe((value) => {
    ownPlayerState.update((state) => {
        state.voter = value
        return state
    })
})

// export const voter: Writable<boolean> = derivedWritableProperty(
//     ownPlayerState,
//     function() { return this.voter },
//     function(value) { this.voter = value; }
// )

export const name: Writable<Option<string>> = writable(get(ownPlayerState).name)
name.subscribe((value) => {
    ownPlayerState.update((state) => {
        state.name = value
        return state
    })
})

export const vote: Writable<Option<string>> = writable(null)
vote.subscribe((value) => client.vote(value))
client.stateChanged.connect((evt) => {
    const player = get(playerId);
    if (!player) {
        console.error("Received state change, but received no player id yet")
        return;
    }
    const newVote = evt.game_state.votes[player]

    if (newVote === null) {
        vote.set(null)
    }
})

export const gameState: Readable<Option<GameState>> =
    (function createRoomState() {
        const { subscribe, set } = writable<Option<GameState>>(null)

        client.joined.connect((evt) => {
            set(evt.state)
        })

        client.state.subscribe((value) => {
            if (value === 'outside' || value === 'connecting') {
                set(null)
            }
        })

        client.stateChanged.connect((evt) => {
            set(evt.game_state)
        })

        return {
            subscribe,
        }
    })()

export const creating_room: Writable<boolean> = writable(false)
export const playerId: Readable<Option<string>> = client.playerId
export const roomId: Readable<Option<string>> = client.roomId
export const lastError: Readable<Option<string>> = client.lastError
export const state: Readable<PlayerState> = client.state

// mutations

// actions

export interface PlayerExtInfo {
    id: string
    name: Option<string>
    voter: boolean
    vote: Option<string>
}

export const players: Readable<Array<PlayerExtInfo>> = (function createRoomState() {
    const { subscribe, set, update } = writable<Array<PlayerExtInfo>>([])

    function findPlayer(state: Array<PlayerExtInfo>, id: string): number {
        return state.findIndex((player) => player.id === id)
    }

    client.joined.connect((evt) => {
        const players: Array<PlayerExtInfo> = []
        for (const player of evt.players) {
            players.push({
                id: player.id,
                name: player.name,
                voter: player.voter,
                vote: evt.state.votes[player.id],
            })
        }
        set(players)
    })

    client.state.subscribe((value) => {
        if (value === 'outside' || value === 'connecting') {
            set([])
        }
    })

    client.playerJoined.connect((evt) => {
        update((players) => {
            const index = findPlayer(players, evt.player.id)
            const info = {
                id: evt.player.id,
                name: evt.player.name,
                voter: evt.player.voter,
                vote: null,
            }

            if (index >= 0) {
                players[index] = info
            } else {
                players.push(info)
            }

            return players
        })
    })

    client.playerChanged.connect((evt) => {
        update((players) => {
            const index = findPlayer(players, evt.player.id)
            if (index >= 0) {
                players[index].name = evt.player.name
                players[index].voter = evt.player.voter
            }
            return players
        })
    })

    client.playerLeft.connect((evt) => {
        update((players) => {
            const index = findPlayer(players, evt.player_id)
            if (index >= 0) {
                players.splice(index, 1)
            }
            return players
        })
    })

    client.stateChanged.connect((evt) => {
        update((players) => {
            for (const [id, vote] of Object.entries(evt.game_state.votes)) {
                const index = findPlayer(players, id)
                if (index >= 0) {
                    players[index].vote = vote
                }
            }
            return players
        })
    })

    return {
        subscribe,
    }
})()

// navigation

client.welcome.connect(() => {
    const state = get(ownPlayerState)
    client.updatePlayer(state.voter, state.name)
})

client.joined.connect((evt) => {
    navigate('/room/' + evt.room)
})

client.rejected.connect(() => {
    navigate('/')
})

// sveltex

export interface Actions {
    changeName(name: string): void
}

export interface Mutations {
    setRemoteName(name: string): void
}
