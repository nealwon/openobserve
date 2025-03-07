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
  <div class="column full-height">
    <DashboardHeader :title="t('dashboard.generalSettingsTitle')" />
    <div>
      <q-form ref="addDashboardForm" @submit="onSubmit">
        <q-input
          v-model="dashboardData.title"
          :label="t('dashboard.name') + ' *'"
          color="input-border"
          bg-color="input-bg"
          class="q-py-md showLabelOnTop"
          stack-label
          outlined
          filled
          dense
          :rules="[(val) => !!val.trim() || t('dashboard.nameRequired')]"
        />
        <span>&nbsp;</span>
        <q-input
          v-model="dashboardData.description"
          :label="t('dashboard.typeDesc')"
          color="input-border"
          bg-color="input-bg"
          class="q-py-md showLabelOnTop"
          stack-label
          outlined
          filled
          dense
        />
        <div class="flex justify-center q-mt-lg">
          <q-btn
            ref="closeBtn"
            v-close-popup="true"
            class="q-mb-md text-bold"
            :label="t('dashboard.cancel')"
            text-color="light-text"
            padding="sm md"
            no-caps
          />
          <q-btn
            :disable="dashboardData.title.trim() === ''"
            :label="t('dashboard.save')"
            class="q-mb-md text-bold no-border q-ml-md"
            color="secondary"
            padding="sm xl"
            type="submit"
            no-caps
            :loading="saveDashboardApi.isLoading.value"
          />
        </div>
      </q-form>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { useStore } from "vuex";
import { reactive } from "vue";
import { getDashboard, updateDashboard } from "@/utils/commons";
import { useRoute } from "vue-router";
import DashboardHeader from "./common/DashboardHeader.vue";
import { useLoading } from "@/composables/useLoading";
import { useQuasar } from "quasar";

export default defineComponent({
  name: "GeneralSettings",
  components: {
    DashboardHeader,
  },
  emits: ["save"],
  setup(props, { emit }) {
    const store: any = useStore();
    const { t } = useI18n();
    const $q = useQuasar();
    const route = useRoute();

    const addDashboardForm: Ref<any> = ref(null);
    const closeBtn: Ref<any> = ref(null);

    const dashboardData = reactive({
      title: "",
      description: "",
    });

    const getDashboardData = async () => {
      const data = await getDashboard(
        store,
        route.query.dashboard,
        route.query.folder ?? "default"
      );
      dashboardData.title = data.title;
      dashboardData.description = data.description;
    };
    onMounted(async () => {
      await getDashboardData();
    });

    const saveDashboardApi = useLoading(async () => {
      // get the latest dashboard data and update the title and description
      const data = JSON.parse(
        JSON.stringify(
          await getDashboard(
            store,
            route.query.dashboard,
            route.query.folder ?? "default"
          )
        )
      );

      // update the values
      data.title = dashboardData.title;
      data.description = dashboardData.description;

      // now lets save it
      await updateDashboard(
        store,
        store.state.selectedOrganization.identifier,
        route.query.dashboard,
        data,
        route?.query?.folder ?? "default"
      );

      $q.notify({
        type: "positive",
        message: "Dashboard updated successfully.",
      });

      emit("save");
    });

    const onSubmit = () => {
      addDashboardForm.value.validate().then((valid: any) => {
        if (!valid) {
          return false;
        }

        saveDashboardApi.execute().catch((err: any) => {
          $q.notify({
            type: "negative",
            message: JSON.stringify(
              err.response.data["error"] || "Dashboard creation failed."
            ),
          });
        });
      });
    };

    return {
      t,
      dashboardData,
      addDashboardForm,
      store,
      saveDashboardApi,
      onSubmit,
      closeBtn,
    };
  },
});
</script>
