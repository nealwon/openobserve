<!-- Copyright 2023 Zinc Labs Inc.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
-->

<template>
  <q-page class="aboutPage">
    <div class="about_head">
      <img
        :src="
          store.state.theme == 'dark'
            ? getImageURL('images/common/open_observe_logo_2.svg')
            : getImageURL('images/common/open_observe_logo.svg')
        "
        class="logo"
        width="177"
      />
      <div class="para">{{ t("about.logoMsg") }}</div>
    </div>

    <q-separator class="separator" />

    <div class="about_data">
      <div class="row head">
        <div class="col-1">{{ t("about.version") }}</div>
        <div class="col-6 col-md-5 col-lg-4">{{ t("about.commitHash") }}</div>
        <div class="col-3 col-md-3 col-lg-2">{{ t("about.buildDate") }}</div>
      </div>
      <div class="row body">
        <div class="col-1">{{ store.state.zoConfig.version }}</div>
        <div class="col-6 col-md-5 col-lg-4">
          {{ store.state.zoConfig.commit_hash }}
        </div>
        <div class="col-3 col-md-3 col-lg-2">
          {{ store.state.zoConfig.build_date }}
        </div>
      </div>
    </div>
  </q-page>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useStore } from "vuex";
import { getImageURL } from "../utils/zincutils";
import { useI18n } from "vue-i18n";

export default defineComponent({
  name: "PageAbout",
  setup() {
    const store = useStore();
    const pageData = ref("Page Data");
    const { t } = useI18n();

    return {
      t,
      store,
      pageData,
      getImageURL,
    };
  },
});
</script>

<style lang="scss" scoped>
.aboutPage {
  padding: 2.25rem 1.5rem;

  .about_head,
  .about_data {
    padding: 0 1.5rem;
  }

  .logo {
    margin-bottom: 0.5rem;
    display: block;
  }

  .para {
    font-weight: 600;
  }

  .q-separator {
    margin: 1.5rem 0;
  }

  .about_data {
    .head {
      margin-bottom: 0.5rem;
      font-weight: 700;
    }

    .body {
      font-weight: 600;
    }
  }
}
</style>
