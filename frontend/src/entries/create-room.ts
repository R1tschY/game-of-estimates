import '../ext/string'
import CreateRoomForm from '../components/CreateRoomForm.svelte'

const createRoomForm = new CreateRoomForm({
    target: document.getElementById('create-room-form')!,
})

export default createRoomForm
