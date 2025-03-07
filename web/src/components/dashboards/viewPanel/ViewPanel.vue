<!-- Copyright 2023 Zinc Labs Inc.

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http:www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License. 
-->

<!-- eslint-disable vue/no-unused-components -->
<template>
  <div style="height: calc(100vh - 57px)" class="scroll">
    <div class="flex justify-between items-center q-pa-md">
      <div class="flex items-center q-table__title q-mr-md">
        <span>
          {{ dashboardPanelData.data.title }}
        </span>
      </div>
      <div class="flex q-gutter-sm items-center">
        <!-- histogram interval for sql queries -->
        <HistogramIntervalDropDown
          v-if="!promqlMode && histogramFields.length"
          v-model="histogramInterval"
          class="q-ml-sm"
          style="width: 150px"
        />

        <DateTimePickerDashboard
          v-model="selectedDate"
          ref="dateTimePickerRef"
        />
        <AutoRefreshInterval
          v-model="refreshInterval"
          trigger
          @trigger="refreshData"
        />
        <q-btn
          class="q-ml-sm"
          outline
          padding="xs"
          no-caps
          icon="refresh"
          @click="refreshData"
        />
        <q-btn
          no-caps
          @click="goBack"
          padding="xs"
          class="q-ml-md"
          flat
          icon="close"
        />
      </div>
    </div>
    <q-separator></q-separator>
    <div class="row" style="height: calc(100vh - 130px); overflow-y: auto">
      <div class="col" style="width: 100%; height: 100%">
        <div class="row" style="height: 100%; overflow-y: auto">
          <div class="col" style="height: 100%">
            <div class="layout-panel-container col" style="height: 100%">
              <VariablesValueSelector
                :variablesConfig="currentDashboardData.data?.variables"
                :selectedTimeDate="dashboardPanelData.meta.dateTime"
                :initialVariableValues="initialVariableValues?.values?.reduce((initialObj: any, variable: any) => {
                  initialObj[variable?.name] = variable?.value;
                  return initialObj;
                }, {})"
                @variablesData="variablesDataUpdated"
              />
              <div style="flex: 1">
                <PanelSchemaRenderer
                  :key="dashboardPanelData.data.type"
                  :panelSchema="chartData"
                  :selectedTimeObj="dashboardPanelData.meta.dateTime"
                  :variablesData="variablesData"
                  :width="6"
                  @error="handleChartApiError"
                  @updated:data-zoom="onDataZoom"
                />
              </div>
              <DashboardErrorsComponent :errors="errorData" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {
  defineComponent,
  ref,
  toRaw,
  nextTick,
  watch,
  reactive,
  onUnmounted,
  onMounted,
} from "vue";

import { useI18n } from "vue-i18n";
import { getDashboard, getPanel } from "../../../utils/commons";
import { useRoute, useRouter } from "vue-router";
import { useStore } from "vuex";
import useDashboardPanelData from "../../../composables/useDashboardPanel";
import DateTimePickerDashboard from "../../../components/DateTimePickerDashboard.vue";
import DashboardErrorsComponent from "../../../components/dashboards/addPanel/DashboardErrors.vue";
import VariablesValueSelector from "../../../components/dashboards/VariablesValueSelector.vue";
import PanelSchemaRenderer from "../../../components/dashboards/PanelSchemaRenderer.vue";
import _ from "lodash-es";
import AutoRefreshInterval from "@/components/AutoRefreshInterval.vue";
import { onActivated } from "vue";
import { parseDuration } from "@/utils/date";
import { Parser } from "node-sql-parser/build/mysql";
import HistogramIntervalDropDown from "@/components/dashboards/addPanel/HistogramIntervalDropDown.vue";

export default defineComponent({
  name: "ViewPanel",
  components: {
    DateTimePickerDashboard,
    DashboardErrorsComponent,
    VariablesValueSelector,
    PanelSchemaRenderer,
    AutoRefreshInterval,
    HistogramIntervalDropDown,
  },
  props: {
    panelId: {
      type: String,
      required: true,
    },
    currentTimeObj: {
      type: Object,
    },
    initialVariableValues: {
      type: Object,
    },
  },
  emits: ["closePanel"],
  setup(props, { emit }) {
    // This will be used to copy the chart data to the chart renderer component
    // This will deep copy the data object without reactivity and pass it on to the chart renderer
    const chartData = ref({});
    const { t } = useI18n();
    const router = useRouter();
    const route = useRoute();
    const store = useStore();
    const { dashboardPanelData, promqlMode, resetDashboardPanelData } =
      useDashboardPanelData();
    // default selected date will be absolute time
    const selectedDate = ref({
      startTime: props?.currentTimeObj?.start_time,
      endTime: props?.currentTimeObj?.end_time,
      valueType: "absolute",
    });
    const dateTimePickerRef: any = ref(null);
    const errorData: any = reactive({
      errors: [],
    });
    let variablesData: any = reactive({});
    const variablesDataUpdated = (data: any) => {
      Object.assign(variablesData, data);
    };
    const currentDashboardData: any = reactive({
      data: {},
    });

    // refresh interval v-model
    const refreshInterval = ref(0);

    // histogram interval
    const histogramInterval: any = ref({
      value: null,
      label: "Auto",
    });

    // array of histogram fields
    let histogramFields: any = ref([]);

    watch(
      () => histogramInterval.value,
      () => {
        // replace the histogram interval in the query by finding histogram aggregation
        dashboardPanelData?.data?.queries?.forEach((query: any) => {
          const parser = new Parser();
          const ast: any = parser.astify(query?.query);

          // Iterate over the columns to check if the column is histogram
          ast.columns.forEach((column: any) => {
            // check if the column is histogram
            if (
              column.expr.type === "function" &&
              column.expr.name === "histogram"
            ) {
              const histogramExpr = column.expr;
              if (
                histogramExpr.args &&
                histogramExpr.args.type === "expr_list"
              ) {
                // if selected histogramInterval is null then remove interval argument
                if (!histogramInterval.value.value) {
                  histogramExpr.args.value = histogramExpr.args.value.slice(
                    0,
                    1
                  );
                }

                // else update interval argument
                else {
                  // check if there is existing interval value
                  // if have then simply update
                  // else insert new arg
                  if (histogramExpr.args.value[1]) {
                    // Update existing interval value
                    histogramExpr.args.value[1] = {
                      type: "single_quote_string",
                      value: `${histogramInterval.value.value}`,
                    };
                  } else {
                    // create new arg for interval
                    histogramExpr.args.value.push({
                      type: "single_quote_string",
                      value: `${histogramInterval.value.value}`,
                    });
                  }
                }
              }
            }
            const sql = parser.sqlify(ast);
            query.query = sql.replace(/`/g, '"');
          });
        });
        // copy the data object excluding the reactivity
        chartData.value = JSON.parse(JSON.stringify(dashboardPanelData.data));
        // refresh the date time based on current time if relative date is selected
        dateTimePickerRef.value && dateTimePickerRef.value.refresh();
      }
    );

    const onDataZoom = (event: any) => {
      const selectedDateObj = {
        start: new Date(event.start),
        end: new Date(event.end),
      };
      // Truncate seconds and milliseconds from the dates
      selectedDateObj.start.setSeconds(0, 0);
      selectedDateObj.end.setSeconds(0, 0);

      // Compare the truncated dates
      if (selectedDateObj.start.getTime() === selectedDateObj.end.getTime()) {
        // Increment the end date by 1 minute
        selectedDateObj.end.setMinutes(selectedDateObj.end.getMinutes() + 1);
      }

      // set it as a absolute time
      dateTimePickerRef?.value?.setCustomDate("absolute", selectedDateObj);
    };

    onUnmounted(async () => {
      // clear a few things
      resetDashboardPanelData();
    });

    onMounted(async () => {
      errorData.errors = [];

      // todo check for the edit more
      if (props.panelId) {
        const panelData = await getPanel(
          store,
          route.query.dashboard,
          props.panelId,
          route.query.folder
        );
        Object.assign(
          dashboardPanelData.data,
          JSON.parse(JSON.stringify(panelData))
        );
        await nextTick();
        chartData.value = JSON.parse(JSON.stringify(dashboardPanelData.data));
      }

      //if sql, get histogram fields from all queries
      histogramFields.value =
        dashboardPanelData.data.queryType != "sql"
          ? []
          : dashboardPanelData.data.queries
              .map((q: any) =>
                [...q.fields.x, ...q.fields.y, ...q.fields.z].find(
                  (f: any) => f.aggregationFunction == "histogram"
                )
              )
              .filter((field: any) => field != undefined);

      // if there is at least 1 histogram field
      // then set the default histogram interval
      if (histogramFields.value.length > 0) {
        for (let i = 0; i < histogramFields.value.length; i++) {
          if (
            histogramFields.value[i]?.args &&
            histogramFields.value[i]?.args[0]?.value
          ) {
            histogramInterval.value = {
              value: histogramFields.value[i]?.args[0]?.value,
              label: histogramFields.value[i]?.args[0]?.value,
            };
            break;
          }
        }
      }
      await nextTick();
      loadDashboard();
    });

    onActivated(() => {
      const params: any = route.query;

      if (params.refresh) {
        refreshInterval.value = parseDuration(params.refresh);
      }
    });

    const refreshData = () => {
      dateTimePickerRef.value.refresh();
    };

    const currentDashboard = toRaw(store.state.currentSelectedDashboard);

    const loadDashboard = async () => {
      let data = JSON.parse(
        JSON.stringify(
          await getDashboard(
            store,
            route.query.dashboard,
            route.query.folder ?? "default"
          )
        )
      );
      currentDashboardData.data = data;

      // if variables data is null, set it to empty list
      if (
        !(
          currentDashboardData.data?.variables &&
          currentDashboardData.data?.variables?.list.length
        )
      ) {
        variablesData.isVariablesLoading = false;
        variablesData.values = [];
      }
    };

    watch(selectedDate, () => {
      updateDateTime(selectedDate.value);
    });

    const updateDateTime = (value: object) => {
      dashboardPanelData.meta.dateTime = {
        start_time: new Date(selectedDate.value.startTime),
        end_time: new Date(selectedDate.value.endTime),
      };
    };
    const goBack = () => {
      emit("closePanel");
    };

    const handleChartApiError = (errorMessage: any) => {
      const errorList = errorData.errors;
      errorList.splice(0);
      errorList.push(errorMessage);
    };
    return {
      t,
      updateDateTime,
      goBack,
      currentDashboard,
      dashboardPanelData,
      chartData,
      selectedDate,
      errorData,
      handleChartApiError,
      variablesDataUpdated,
      currentDashboardData,
      variablesData,
      dateTimePickerRef,
      refreshInterval,
      refreshData,
      promqlMode,
      histogramInterval,
      histogramFields,
      onDataZoom,
    };
  },
});
</script>

<style lang="scss" scoped>
.layout-panel-container {
  display: flex;
  flex-direction: column;
}
</style>
