<script>
    import './main.sass'

    import { Router, Link, Route } from 'svelte-routing'
    import Home from './routes/Home.svelte'
    import Room from './routes/Room.svelte'

    export let url = ''
</script>

<Router {url}>
    <div>
        <Route path="/">
            <Home />
        </Route>
        <Route path="room/:id" let:params>
            <Room id={params.id} />
        </Route>
    </div>
</Router>
