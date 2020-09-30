import { writable } from 'svelte/store'
import { navigate } from 'svelte-routing'

let socket = null

// state

export const connected = writable(false)
export const player_id = writable(null)

export const vote = writable(null)
vote.subscribe((value) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
        // TODO: check: within a room?
        socket.send(JSON.stringify({ type: 'Vote', vote: value }))
    }
})

export const creating_room = writable(false)

// mutations

// actions

export const game = (function createRoomState() {
    const { subscribe, set, update } = writable({
        id: null,
        status: 'outside',
        last_error: null,
        players: [],
        state: null,
    })

    return {
        subscribe,

        create: () => {
            console.log('Trying to create game')
            if (socket && socket.readyState === WebSocket.OPEN) {
                socket.send(JSON.stringify({ type: 'CreateGame' }))
                set({
                    id: null,
                    status: 'creating',
                    last_error: null,
                    players: [],
                    state: null,
                })
            } else {
                update((game) => {
                    game.status = 'outside'
                    game.last_error = 'disconnected'
                    return game
                })
            }
        },

        join: (id) => {
            console.log('Trying to join ' + id)
            if (socket && socket.readyState === WebSocket.OPEN) {
                update((game) => {
                    if (game.status === 'joined' && game.id === id) {
                        return game
                    }
                    socket.send(JSON.stringify({ type: 'JoinGame', game: id }))
                    return {
                        id: null,
                        status: 'joining',
                        last_error: null,
                        players: [],
                        state: null,
                    }
                })
            } else {
                update((game) => {
                    game.id = id
                    game.status = 'outside'
                    game.last_error = 'disconnected'
                    return game
                })
            }
        },

        on_welcome: (player_id) => {
            update((state) => {
                if (state.id !== null) {
                    game.join(state.id)
                }
                return state
            })
        },

        // TODO: connect to websocket on your own
        on_joined: (data) => {
            update((game) => {
                if (game.id === null) navigate('/room/' + data.game)
                return {
                    id: data.game,
                    status: 'joined',
                    last_error: null,
                    players: data.players,
                    state: data.state,
                }
            })
        },

        // TODO: connect to websocket on your own
        on_disconnected: () => {
            update((game) => {
                if (game.state !== 'outside') {
                    game.status = 'outside'
                    game.last_error = 'disconnected'
                }
                return game
            })
        },

        // TODO: connect to websocket on your own
        on_error: () => {
            update((game) => {
                if (game.state !== 'outside') {
                    game.status = 'outside'
                    game.last_error = 'error'
                }
                return game
            })
        },

        on_player_joined: (player) => {
            update((game) => {
                game.players.push(player)
                if (player.voter) {
                    game.state.votes[player_id] = null
                }
                return game
            })
        },

        on_player_changed: (player) => {
            update((game) => {
                let index = game.players.findIndex((p) => p.id == player.id)
                if (index !== -1) {
                    game.players[index] = player
                }
                return game
            })
        },

        on_player_left: (player_id) => {
            update((game) => {
                let index = game.players.findIndex((p) => p.id == player_id)
                if (index !== -1) {
                    game.players.splice(index, 1)
                }
                delete game.state.votes[player_id]
                return game
            })
        },

        on_state_changed: (state) => {
            update((game) => {
                game.state = state
                return game
            })
        },
    }
})()

function on_connected(event) {
    console.log('Connected', event)
    connected.set(true)
}

function on_disconnected(event) {
    console.log('Disconnected', event)
    connected.set(false)
    game.on_disconnected()
}

function on_connection_error(event) {
    console.log('Error', event)
    connected.set(false)
    game.on_error()
}

function on_message_arrived(event) {
    console.log('Got message', event)
    let data = JSON.parse(event.data)
    switch (data.type) {
        case 'Welcome':
            console.debug('Welcome message')
            player_id.set(data.player_id)
            game.on_welcome(data.player_id)
            break

        case 'Joined':
            console.debug('Joined')
            game.on_joined(data)
            break

        case 'PlayerJoined':
            game.on_player_joined(data.player)
            break

        case 'PlayerChanged':
            game.on_player_changed(data.player)
            break

        case 'PlayerLeft':
            game.on_player_left(data.player_id)
            break

        case 'GameChanged':
            game.on_state_changed(data.game_state)
            break

        default:
            console.debug('Unknown message', data)
            break
    }
}

function connectWs() {
    socket = new WebSocket('ws://localhost:5500')
    socket.addEventListener('open', on_connected)
    socket.addEventListener('message', on_message_arrived)
    socket.addEventListener('close', on_disconnected)
    socket.addEventListener('error', on_connection_error)
}

// init

connectWs()
