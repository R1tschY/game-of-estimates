import { derived, get, Readable, Writable, writable } from 'svelte/store'
import { navigate } from 'svelte-routing'
import { client, GameState, OwnPlayerState, PlayerState, wsService } from './client'
import type { Option } from './basetypes'

// state

export const connected: Readable<boolean> = wsService.connected_store
export const connecting: Readable<boolean> = wsService.connecting_store
export const player_id: Readable<Option<string>> = writable(null)
export const voter: Writable<boolean> = writable(true)
export const name: Writable<Option<string>> = writable(null)

export const ownPlayerState: Writable<OwnPlayerState> = writable({
    voter: true,
    name: null
})
ownPlayerState.subscribe((value) => {
    client.updatePlayer(value.voter, value.name)
})
voter.subscribe((value) => {
    ownPlayerState.update((player) => {
        player.voter = value;
        return player
    })
})
name.subscribe((value) => {
    ownPlayerState.update((player) => {
        player.name = value;
        return player
    })
})

// function derived_writable<T, U>(store: Writable<U>, read: (value: U) => T, update: (old: U, value: T) => U): Writable<T> {
//     let newStore = derived(store, read)
//     return {
//         subscribe: newStore.subscribe,
//         set: function(value) {
//             store.update((old) => update(old, value))
//         },
//         update: function(fn) {
//             store.update((old) => update(old, fn(read(old))))
//         }
//     }
// }

export const vote: Writable<Option<string>> = writable(null)
vote.subscribe((value) => client.vote(value))

export const creating_room: Writable<boolean> = writable(false)

// mutations

// actions

interface RoomState {
    id: Option<string>,
    status: 'outside' | 'joined',
    last_error: Option<string>,
    players: PlayerState[],
    state: Option<GameState>,
}

function initRoomState(): RoomState {
    return {
        id: null,
        status: 'outside',
        last_error: null,
        players: [],
        state: null,
    }
}

export const room: Readable<RoomState> = (function createRoomState() {
    const { subscribe, set, update } = writable(initRoomState())

    client.welcome.connect(evt => {
        update((state) => {
            if (state.id !== null) {
                client.joinRoom(state.id)
            }
            return state
        })
    })

    client.joined.connect(evt => {
        update((room) => {
            if (room.id !== evt.room)
                navigate('/room/' + evt.room)
            return {
                id: evt.room,
                status: 'joined',
                last_error: null,
                players: evt.players,
                state: evt.state,
            }
        })
    })

    client.rejected.connect(evt => {
        navigate('/')
        update((room) => {
            let state = initRoomState()
            state.last_error = "room does not exist"
            return state
        })
    })

    wsService.disconnected.connect(() => {
        update((room) => {
            if (room.status !== 'outside') {
                room.status = 'outside'
                room.last_error = 'disconnected'
            }
            return room
        })
    })

    wsService.error.connect(() => {
        update((room) => {
            if (room.status !== 'outside') {
                room.status = 'outside'
                room.last_error = 'error'
            }
            return room
        })
    })

    client.playerJoined.connect(evt => {
        update((room) => {
            room.players.push(evt.player)
            if (evt.player.voter) {
                room.state.votes[evt.player.id] = null
            }
            return room
        })
    })

    client.playerChanged.connect(evt => {
        update((room) => {
            let index = room.players.findIndex((p) => p.id == evt.player.id)
            if (index !== -1) {
                room.players[index] = evt.player
            }
            return room
        })
    })

    client.playerLeft.connect(evt => {
        update((room) => {
            let pid = get(player_id)
            let index = room.players.findIndex((p) => p.id == pid)
            if (index !== -1) {
                room.players.splice(index, 1)
            }
            delete room.state.votes[evt.player_id]
            return room
        })
    })

    client.stateChanged.connect(evt => {
        update((room) => {
            room.state = evt.game_state
            return room
        })
    })

    return {
        subscribe
    }
})()
