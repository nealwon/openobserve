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
  <q-page class="q-pa-lg">
    <div
      v-if="!no_data_ingest"
      class="q-pa-md row items-start q-gutter-md"
      style="margin: 0 auto; justify-content: center"
    >
      <q-card class="my-card">
        <q-card-section align="center" flat bordered class="my-card">
          <div class="text-subtitle1">{{ t("home.streams") }}</div>
          <div class="text-h6">{{ summary.streams_count }}</div>
          <div class="text-subtitle1">{{ t("home.totalDataIngested") }}</div>
          <div class="text-h6">{{ summary.ingested_data }}</div>
        </q-card-section>

        <q-separator />

        <q-card-actions align="center">
          <q-btn no-caps color="primary" flat
            >{{ t("home.view") }}
            <router-link
              exact
              :to="{ name: 'logstreams' }"
              class="absolute full-width full-height"
            ></router-link>
          </q-btn>
        </q-card-actions>
      </q-card>

      <q-card align="center" class="my-card">
        <q-card-section align="center" flat bordered class="my-card">
          <div class="text-subtitle1">{{ t("home.queryFunctions") }}</div>
          <div class="text-h6">{{ summary.query_fns }}</div>
          <div class="text-subtitle1">{{ t("home.ingestFunctions") }}</div>
          <div class="text-h6">{{ summary.ingest_fns }}</div>
        </q-card-section>
        <q-separator />
        <q-card-actions align="center">
          <q-btn no-caps color="primary" flat
            >{{ t("home.view") }}
            <router-link
              exact
              :to="{ name: 'functions' }"
              class="absolute full-width full-height"
            ></router-link>
          </q-btn>
        </q-card-actions>
      </q-card>

      <q-card class="my-card">
        <q-card-section align="center" flat bordered class="my-card">
          <div class="text-subtitle1">{{ t("home.scheduledAlert") }}</div>
          <div class="text-h6">{{ summary.scheduled_alerts }}</div>
          <div class="text-subtitle1">{{ t("home.rtAlert") }}</div>
          <div class="text-h6">{{ summary.rt_alerts }}</div>
        </q-card-section>
        <q-separator />
        <q-card-actions align="center">
          <q-btn no-caps color="primary" flat
            >{{ t("home.view") }}
            <router-link
              exact
              :to="{ name: 'alerts' }"
              class="absolute full-width full-height"
            ></router-link>
          </q-btn>
        </q-card-actions>
      </q-card>
    </div>

    <div
      v-if="no_data_ingest"
      class="q-pa-md row items-start q-gutter-md"
      style="margin: 0 auto; justify-content: center"
    >
      <q-card class="my-card">
        <q-card-section align="center" flat bordered class="my-card">
          <div class="text-h6">{{ t("home.noData") }}</div>
          <div class="text-subtitle1">{{ t("home.ingestionMsg") }}</div>
        </q-card-section>

        <q-separator />

        <q-card-actions align="center">
          <q-btn
            no-caps
            color="primary"
            @click="() => $router.push({ name: 'ingestion' })"
            flat
            >{{ t("home.findIngestion") }}
          </q-btn>
        </q-card-actions>
      </q-card>
    </div>
    <div class="row justify-center items-center">
      <video width="400" height="225" controls>
        <source
          src="https://videos.openobserve.ai/OpenObserve_Introduction.mp4"
          type="video/mp4"
        />
        Your browser does not support the video tag.
      </video>
    </div>
  </q-page>
</template>

<script lang="ts">
import { useQuasar } from "quasar";
import { defineComponent, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useStore } from "vuex";
import orgService from "../services/organizations";
import config from "../aws-exports";
import { formatSizeFromMB } from "@/utils/zincutils";
export default defineComponent({
  name: "PageHome",

  setup() {
    const store = useStore();
    const { t } = useI18n();
    const summary: any = ref([]);
    const $q = useQuasar();
    const no_data_ingest = ref(false);
    const isCloud = config.isCloud;
    const getSummary = (org_id: any) => {
      const dismiss = $q.notify({
        spinner: true,
        message: "Please wait while loading summary...",
      });
      orgService
        .get_organization_summary(org_id)
        .then((res) => {
          if (
            res.data.streams.length == 0 &&
            res.data.functions.length == 0 &&
            res.data.alerts.length == 0
          ) {
            no_data_ingest.value = true;
            summary.value = {};
            dismiss();
            return;
          }
          let sum = 0;
          if (res.data.streams.length > 0) {
            sum = res.data.streams.reduce(
              (
                acc: number,
                val: {
                  [x: string]: any;
                  storage_size: any;
                }
              ) => {
                return acc + val.stats.storage_size;
              },
              0
            );
          }

          let ingest_fns = 0;
          let query_fns = 0;
          if (res.data.functions.length > 0) {
            res.data.functions.forEach((fn: { stream_name: any }) => {
              if (fn.stream_name && fn.stream_name != "") {
                ingest_fns += 1;
              } else {
                query_fns += 1;
              }
            });
          }

          let rt_alerts = 0;
          let scheduled_alerts = 0;
          if (res.data.alerts.length > 0) {
            res.data.alerts.forEach((alert: { is_real_time: any }) => {
              if (alert.is_real_time) {
                rt_alerts += 1;
              } else {
                scheduled_alerts += 1;
              }
            });
          }
          summary.value = {
            streams_count: res.data.streams.length,
            ingested_data: formatSizeFromMB(sum.toFixed(2)),
            ingest_fns: ingest_fns,
            query_fns: query_fns,
            rt_alerts: rt_alerts,
            scheduled_alerts: scheduled_alerts,
          };
          no_data_ingest.value = false;
          dismiss();
        })
        .catch((err) => {
          console.log(err);
          dismiss();
          $q.notify({
            type: "negative",
            message: "Error while pulling summary.",
            timeout: 2000,
          });
        });
    };

    getSummary(store.state.selectedOrganization.identifier);

    return {
      t,
      store,
      summary,
      no_data_ingest,
      getSummary,
      isCloud,
    };
  },
  computed: {
    selectedOrg() {
      return this.store.state.selectedOrganization?.identifier;
    },
  },
  watch: {
    selectedOrg(newVal: any, oldVal: any) {
      if (newVal != oldVal || this.summary.value == undefined) {
        this.summary = {};
        this.getSummary(this.store.state?.selectedOrganization?.identifier);
      }
    },
  },
});
</script>

<style scoped lang="scss">
.pointer-to-demo {
  margin-left: auto;
  margin-right: 0;
}

.pointer-description {
  display: flex;
  align-items: center;
}

.my-card {
  background-color: rgba(0, 0, 0, 0.045);
}
</style>
