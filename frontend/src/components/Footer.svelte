<script lang="ts">
  import GitHub from '../icons/GitHub.svelte'
  import {getText} from "../i18n";
</script>

<style lang="scss">
  .footer {
      border-top: 1px solid hsl(0, 0%, 86%);
      background-color: hsl(0, 0%, 99%);
  }
</style>

<footer class="footer">
  <div class="content has-text-centered">
    <p>
      {@html getText("byAuthor").format("<strong>Game Of Estimates</strong>", "<a href=\"https://github.com/R1tschY\">Richard Liebscher</a>")}
    </p>
    <p>
      {@html getText("license").format("<a href=\"http://opensource.org/licenses/mit-license.php\">MIT</a>")}
      <div>
        <a href="https://github.com/R1tschY/game_of_estimates"><span><GitHub class="footer-icon"/></span></a>
      </div>
  </div>
</footer>