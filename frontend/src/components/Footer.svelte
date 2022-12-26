<script lang="ts">
    import GitHub from '../svg/GitHub.svg?raw'
    import { getText } from '../i18n'
</script>

<footer class="footer">
    <div class="content has-text-centered">
        <p>
            {@html getText('byAuthor').format(
                '<strong>Game Of Estimates</strong>',
                '<a href="https://github.com/R1tschY">Richard Liebscher</a>',
            )}
        </p>
        <p>
            {@html getText('license').format(
                '<a href="https:///opensource.org/licenses/mit-license.php">MIT</a>',
            )}
        </p>
        <div>
            <a
                href="https://github.com/R1tschY/game_of_estimates"
                class="footer-icon"
            >
                {@html GitHub}
            </a>
        </div>
    </div>
</footer>

<style lang="scss">
    .footer {
        border-top: 1px solid hsl(0, 0%, 86%);
        background-color: hsl(0, 0%, 99%);
    }
</style>
