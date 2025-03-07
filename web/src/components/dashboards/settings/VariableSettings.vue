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
  <div>
    <div v-if="isAddVariable" class="column full-height">
      <AddSettingVariable
        v-if="isAddVariable"
        @save="handleSaveVariable"
        :variableName="selectedVariable"
        @close="goBackToDashboardList"
      />
    </div>
    <div v-else class="column full-height">
      <DashboardHeader title="Variables">
        <template #right>
          <q-btn
            class="text-bold no-border q-ml-md"
            no-caps
            no-outline
            rounded
            color="secondary"
            :label="t(`dashboard.newVariable`)"
            @click="addVariables"
          />
        </template>
      </DashboardHeader>
      <div>
        <q-table
          ref="qTable"
          :rows="dashboardVariableData?.data"
          :columns="columns"
          row-key="name"
          :pagination="pagination"
        >
          <template #no-data>
            <NoData />
          </template>
          <!-- add delete icon in actions column -->
          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn
                icon="edit"
                class="q-ml-xs"
                padding="sm"
                unelevated
                size="sm"
                round
                flat
                :title="t('dashboard.edit')"
                @click="editVariableFn(props.row.name)"
              ></q-btn>
              <q-btn
                :icon="outlinedDelete"
                :title="t('dashboard.delete')"
                class="q-ml-xs"
                padding="sm"
                unelevated
                size="sm"
                round
                flat
                @click.stop="showDeleteDialogFn(props)"
              ></q-btn>
            </q-td>
          </template>
        </q-table>
        <ConfirmDialog
          title="Delete Variable"
          message="Are you sure you want to delete the variable?"
          @update:ok="deleteVariableFn"
          @update:cancel="confirmDeleteDialog = false"
          v-model="confirmDeleteDialog"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {
  defineComponent,
  ref,
  onMounted,
  onActivated,
  reactive,
  toRef,
} from "vue";
import { useI18n } from "vue-i18n";
import { useStore } from "vuex";
import { useRoute } from "vue-router";
import { getImageURL } from "../../../utils/zincutils";
import { getDashboard, deleteVariable } from "../../../utils/commons";
import AddSettingVariable from "./AddSettingVariable.vue";
import DashboardHeader from "./common/DashboardHeader.vue";
import { outlinedDelete } from "@quasar/extras/material-icons-outlined";
import NoData from "../../shared/grid/NoData.vue";
import ConfirmDialog from "../../ConfirmDialog.vue";
import type { QTableProps } from "quasar";

export default defineComponent({
  name: "VariableSettings",
  components: {
    AddSettingVariable,
    NoData,
    ConfirmDialog,
    DashboardHeader,
  },
  emits: ["save"],
  setup(props, { emit }) {
    const store: any = useStore();
    const beingUpdated: any = ref(false);
    const addDashboardForm: any = ref(null);
    const disableColor: any = ref("");
    const isValidIdentifier: any = ref(true);
    const { t } = useI18n();
    const route = useRoute();
    const isAddVariable = ref(false);
    const dashboardVariableData = reactive({
      data: [],
    });
    const pagination: any = ref({
      rowsPerPage: 20,
    });
    const selectedVariable = ref(null);
    const confirmDeleteDialog = ref<boolean>(false);
    const selectedDelete: any = ref(null);
    const columns = ref<QTableProps["columns"]>([
      {
        name: "#",
        label: "#",
        field: "#",
        align: "left",
        style: "width: 70px",
      },
      {
        name: "name",
        field: "name",
        label: t("dashboard.name"),
        align: "left",
        sortable: true,
      },
      {
        name: "actions",
        field: "actions",
        label: t("dashboard.actions"),
        align: "center",
        sortable: false,
        style: "width: 110px",
      },
    ]);

    onMounted(async () => {
      await getDashboardData();
    });

    onActivated(async () => {
      await getDashboardData();
    });

    const getDashboardData = async () => {
      const data = JSON.parse(
        JSON.stringify(
          await getDashboard(
            store,
            route.query.dashboard,
            route.query.folder ?? "default"
          )
        )
      )?.variables?.list;
      dashboardVariableData.data = (data || []).map(
        (it: any, index: number) => {
          return {
            "#": index < 9 ? `0${index + 1}` : index + 1,
            name: it.name,
          };
        }
      );
    };

    const addVariables = () => {
      selectedVariable.value = null;
      isAddVariable.value = true;
    };

    const showDeleteDialogFn = (props: any) => {
      selectedDelete.value = props.row;
      confirmDeleteDialog.value = true;
    };

    const deleteVariableFn = async () => {
      if (selectedDelete.value) {
        const variableName = selectedDelete?.value?.name;

        await deleteVariable(
          store,
          route.query.dashboard,
          variableName,
          route.query.folder ?? "default"
        );

        await getDashboardData();
        emit("save");
      }
    };
    const handleSaveVariable = async () => {
      isAddVariable.value = false;
      await getDashboardData();
      emit("save");
    };
    const goBackToDashboardList = () => {
      isAddVariable.value = false;
    };
    const editVariableFn = async (name: any) => {
      selectedVariable.value = name;

      isAddVariable.value = true;
    };

    return {
      t,
      disableColor,
      isPwd: ref(true),
      beingUpdated,
      status,
      addDashboardForm,
      store,
      isValidIdentifier,
      getImageURL,
      getDashboardData,
      addVariables,
      dashboardVariableData,
      isAddVariable,
      columns,
      pagination,
      outlinedDelete,
      showDeleteDialogFn,
      confirmDeleteDialog,
      deleteVariableFn,
      goBackToDashboardList,
      editVariableFn,
      selectedVariable,
      handleSaveVariable,
    };
  },
});
</script>
