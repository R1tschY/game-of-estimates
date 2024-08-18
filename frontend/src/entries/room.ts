import '../ext/string'
import RoomContainer from '../components/RoomContainer.svelte'

const target = document.getElementById('room-container')!

const roomContainer = new RoomContainer({
    target,
    props: {
        id: target.dataset.roomid!,
    },
})

export default roomContainer
