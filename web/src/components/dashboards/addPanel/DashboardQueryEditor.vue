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
  <div class="col-auto" data-test="dashboard-panel-searchbar">
    <q-bar
      class="row sql-bar"
      style="display: flex; justify-content: space-between"
      @click.stop="onDropDownClick"
    >
      <div
        style="display: flex; flex-direction: row; align-items: center"
        data-test="dashboard-query-data"
      >
        <div>
          <q-icon
            flat
            :name="
              !dashboardPanelData.layout.showQueryBar
                ? 'arrow_right'
                : 'arrow_drop_down'
            "
            text-color="black"
            class="q-mr-sm"
          />
        </div>
        <q-space />
        <div style="max-width: 600px">
          <q-tabs
            v-if="promqlMode || dashboardPanelData.data.type == 'geomap'"
            v-model="dashboardPanelData.layout.currentQueryIndex"
            narrow-indicator
            dense
            inline-label
            outside-arrows
            mobile-arrows
            @click.stop
          >
            <q-tab
              no-caps
              :ripple="false"
              v-for="(tab, index) in dashboardPanelData.data.queries"
              :key="index"
              :name="index"
              :label="'Query ' + (index + 1)"
              @click.stop
            >
              <q-icon
                v-if="
                  index > 0 ||
                  (index === 0 && dashboardPanelData.data.queries.length > 1)
                "
                name="close"
                class="q-ml-sm"
                @click.stop="removeTab(index)"
                style="cursor: pointer"
              />
            </q-tab>
          </q-tabs>
          <!-- <div v-if="promqlMode" class="query-tabs-container">
                    <div v-for="(tab, index) in dashboardPanelData.data.queries" :key="index" class="query-tab" :class="{ 'active': index === activeTab }" @click="handleActiveTab(index)">
                        <div class="tab-label">{{ 'Query ' + (index + 1) }}</div>
                        <div v-if="index > 0 || (index === 0 && dashboardPanelData.data.queries.length > 1)" @click.stop="removeTab(index)">
                            <i class="material-icons">cancel</i>
                        </div>
                    </div>
                </div> -->
        </div>
        <span
          v-if="!(promqlMode || dashboardPanelData.data.type == 'geomap')"
          class="text-subtitle2 text-weight-bold"
          >{{ t("panel.sql") }}</span
        >
        <q-btn
          v-if="promqlMode || dashboardPanelData.data.type == 'geomap'"
          round
          flat
          @click.stop="addTab"
          icon="add"
          style="margin-right: 10px"
        ></q-btn>
      </div>
      <div>
        <QueryTypeSelector></QueryTypeSelector>
      </div>
    </q-bar>
  </div>
  <div
    class="col"
    :style="
      !dashboardPanelData.layout.showQueryBar ? 'height: 0px;' : 'height: auto;'
    "
    style="overflow: hidden"
    data-test="dashboard-query"
  >
    <div class="column" style="width: 100%; height: 100%">
      <div class="col" style="width: 100%">
        <query-editor
          ref="queryEditorRef"
          class="monaco-editor"
          v-model:query="currentQuery"
          data-test="dashboard-panel-query-editor"
          v-model:functions="dashboardPanelData.meta.stream.functions"
          v-model:fields="dashboardPanelData.meta.stream.selectedStreamFields"
          :keywords="
            dashboardPanelData.data.queryType === 'promql'
              ? autoCompletePromqlKeywords
              : []
          "
          @update-query="updateQuery"
          @run-query="searchData"
          :readOnly="
            !dashboardPanelData.data.queries[
              dashboardPanelData.layout.currentQueryIndex
            ].customQuery
          "
          :language="dashboardPanelData.data.queryType"
        ></query-editor>
      </div>
      <div style="color: red; z-index: 100000" class="q-mx-sm col-auto">
        {{ dashboardPanelData.meta.errors.queryErrors.join(", ") }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
// @ts-nocheck
import {
  defineComponent,
  ref,
  watch,
  reactive,
  toRaw,
  onActivated,
  computed,
  onMounted,
} from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useQuasar } from "quasar";
import { Parser } from "node-sql-parser/build/mysql";
import ConfirmDialog from "../../../components/ConfirmDialog.vue";
import QueryEditor from "../QueryEditor.vue";
import useDashboardPanelData from "../../../composables/useDashboardPanel";
import QueryTypeSelector from "../addPanel/QueryTypeSelector.vue";
import usePromqlSuggestions from "@/composables/usePromqlSuggestions";

export default defineComponent({
  name: "DashboardQueryEditor",
  components: {
    QueryEditor,
    ConfirmDialog,
    QueryTypeSelector,
  },
  emits: ["searchdata"],
  methods: {
    searchData() {
      this.$emit("searchdata");
    },
  },
  setup() {
    const router = useRouter();
    const { t } = useI18n();
    const $q = useQuasar();
    const {
      dashboardPanelData,
      promqlMode,
      updateXYFieldsOnCustomQueryChange,
      addQuery,
      removeQuery,
    } = useDashboardPanelData();
    const confirmQueryModeChangeDialog = ref(false);
    const parser = new Parser();
    let streamName = "";
    const { autoCompleteData, autoCompletePromqlKeywords, getSuggestions } =
      usePromqlSuggestions();
    const queryEditorRef = ref(null);

    const addTab = () => {
      addQuery();
      dashboardPanelData.layout.currentQueryIndex =
        dashboardPanelData.data.queries.length - 1;
    };
    const updateQuery = (query, fields) => {
      if (dashboardPanelData.data.queryType === "promql") {
        updatePromQLQuery(query, fields);
      }
    };

    const removeTab = async (index) => {
      if (
        dashboardPanelData.layout.currentQueryIndex >=
        dashboardPanelData.data.queries.length - 1
      )
        dashboardPanelData.layout.currentQueryIndex -= 1;
      removeQuery(index);
    };

    const currentQuery = computed({
      get: () => {
        return promqlMode.value
          ? dashboardPanelData.data.queries[
              dashboardPanelData.layout.currentQueryIndex
            ].query
          : dashboardPanelData.data.queries[
              dashboardPanelData.layout.currentQueryIndex
            ].query;
      },
      set: (value) => {
        if (promqlMode.value) {
          dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].query = value;
        } else {
          dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].query = value;
        }
      },
    });

    // toggle show query view
    const onDropDownClick = () => {
      dashboardPanelData.layout.showQueryBar =
        !dashboardPanelData.layout.showQueryBar;
    };

    watch(
      () => dashboardPanelData.layout.showQueryBar,
      () => {
        window.dispatchEvent(new Event("resize"));
      }
    );

    onMounted(() => {
      dashboardPanelData.meta.errors.queryErrors = [];
    });

    let query = "";
    // Generate the query when the fields are updated
    watch(
      () => [
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.stream,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.x,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.y,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.z,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.filter,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].customQuery,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.latitude,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.longitude,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.weight,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].config.limit,
      ],
      () => {
        // only continue if current mode is auto query generation
        if (
          !dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].customQuery
        ) {
          if (dashboardPanelData.data.type == "geomap") {
            query = geoMapChart();
          } else {
            query = sqlchart();
          }
          dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].query = query;
        }
      },
      { deep: true }
    );

    const geoMapChart = () => {
      let query = "";

      const { latitude, longitude, weight } =
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields;

      if (latitude && longitude) {
        query += `SELECT ${latitude.column} as ${latitude.alias}, ${longitude.column} as ${longitude.alias}`;
      } else if (latitude) {
        query += `SELECT ${latitude.column} as ${latitude.alias}`;
      } else if (longitude) {
        query += `SELECT ${longitude.column} as ${longitude.alias}`;
      }

      if (query) {
        if (weight) {
          const weightField = weight.aggregationFunction
            ? weight.aggregationFunction == "count-distinct"
              ? `count(distinct(${weight.column}))`
              : `${weight.aggregationFunction}(${weight.column})`
            : `${weight.column}`;
          query += `, ${weightField} as ${weight.alias}`;
        }
        query += ` FROM "${
          dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].fields.stream
        }" `;
        if (weight && weight.aggregationFunction) {
          if (latitude || longitude) {
            const aliases = [latitude?.alias, longitude?.alias]
              .filter(Boolean)
              .join(", ");
            query += `GROUP BY ${aliases}`;
          }
        }
      }

      // array of sorting fields with followed by asc or desc
      const orderByArr = [];

      [latitude, longitude, weight].forEach((it: any) => {
        // ignore if None is selected or sortBy is not there
        if (it?.sortBy) {
          orderByArr.push(`${it.alias} ${it.sortBy}`);
        }
      });

      // append with query by joining array with comma
      query += orderByArr.length ? " ORDER BY " + orderByArr.join(", ") : "";

      // append limit
      // if limit is less than or equal to 0 then don't add
      const queryLimit =
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].config.limit ?? 0;
      query += queryLimit > 0 ? " LIMIT " + queryLimit : "";

      return query;
    };

    const sqlchart = () => {
      // STEP 1: first check if there is at least 1 field selected
      if (
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.x.length == 0 &&
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.y.length == 0 &&
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.z.length == 0
      ) {
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].query = "";
        return;
      }

      // STEP 2: Now, continue if we have at least 1 field selected
      // merge the fields list
      let query = "SELECT ";
      const fields = [
        ...dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.x,
        ...dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.y,
        ...(dashboardPanelData.data?.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields?.z
          ? [
              ...dashboardPanelData.data.queries[
                dashboardPanelData.layout.currentQueryIndex
              ].fields.z,
            ]
          : []),
      ].flat();
      const filter = [
        ...dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields?.filter,
      ];
      const array = fields.map((field, i) => {
        let selector = "";

        // TODO: add aggregator
        if (field.aggregationFunction) {
          switch (field.aggregationFunction) {
            case "count-distinct":
              selector += `count(distinct(${field.column}))`;
              break;
            case "histogram": {
              // if inteval is not null, then use it
              if (field?.args && field?.args?.length && field?.args[0].value) {
                selector += `${field.aggregationFunction}(${field.column}, '${field.args[0].value}')`;
              } else {
                selector += `${field.aggregationFunction}(${field.column})`;
              }
              break;
            }
            default:
              selector += `${field.aggregationFunction}(${field.column})`;
              break;
          }
        } else {
          selector += `${field.column}`;
        }
        selector += ` as "${field.alias}"${
          i == fields.length - 1 ? " " : ", "
        }`;
        return selector;
      });
      query += array.join("");

      // now add from stream name
      query += ` FROM "${
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].fields.stream
      }" `;

      const filterData = filter?.map((field, i) => {
        let selectFilter = "";
        if (field.type == "list" && field.values?.length > 0) {
          selectFilter += `${field.column} IN (${field.values
            .map((it) => `'${it}'`)
            .join(", ")})`;
        } else if (field.type == "condition" && field.operator != null) {
          selectFilter += `${field.column} `;
          if (["Is Null", "Is Not Null"].includes(field.operator)) {
            switch (field.operator) {
              case "Is Null":
                selectFilter += `IS NULL`;
                break;
              case "Is Not Null":
                selectFilter += `IS NOT NULL`;
                break;
            }
          } else if (field.value != null && field.value != "") {
            switch (field.operator) {
              case "=":
              case "<>":
              case "<":
              case ">":
              case "<=":
              case ">=":
                selectFilter += `${field.operator} ${field.value}`;
                break;
              case "Contains":
                selectFilter += `LIKE '%${field.value}%'`;
                break;
              case "Not Contains":
                selectFilter += `NOT LIKE '%${field.value}%'`;
                break;
              default:
                selectFilter += `${field.operator} ${field.value}`;
                break;
            }
          }
        }
        return selectFilter;
      });
      const filterItems = filterData.filter((it: any) => it);
      if (filterItems.length > 0) {
        query += "WHERE " + filterItems.join(" AND ");
      }

      // add group by statement
      const xAxisAlias = dashboardPanelData.data.queries[
        dashboardPanelData.layout.currentQueryIndex
      ].fields.x.map((it: any) => it.alias);
      const yAxisAlias = dashboardPanelData.data.queries[
        dashboardPanelData.layout.currentQueryIndex
      ].fields.y.map((it: any) => it.alias);

      if (dashboardPanelData.data.type == "heatmap") {
        query +=
          xAxisAlias.length && yAxisAlias.length
            ? " GROUP BY " +
              xAxisAlias.join(", ") +
              ", " +
              yAxisAlias.join(", ")
            : "";
      } else {
        query += xAxisAlias.length ? " GROUP BY " + xAxisAlias.join(", ") : "";
      }

      // array of sorting fields with followed by asc or desc
      const orderByArr = [];

      fields.forEach((it: any) => {
        // ignore if None is selected or sortBy is not there
        if (it?.sortBy) {
          orderByArr.push(`${it.alias} ${it.sortBy}`);
        }
      });

      // append with query by joining array with comma
      query += orderByArr.length ? " ORDER BY " + orderByArr.join(", ") : "";

      // append limit
      // if limit is less than or equal to 0 then don't add
      const queryLimit =
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].config.limit ?? 0;
      query += queryLimit > 0 ? " LIMIT " + queryLimit : "";

      return query;
    };

    watch(
      () => [
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].query,
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].customQuery,
        dashboardPanelData.meta.stream.selectedStreamFields,
      ],
      () => {
        // Only continue if the current mode is "show custom query"
        if (
          dashboardPanelData.data.queries[
            dashboardPanelData.layout.currentQueryIndex
          ].customQuery &&
          dashboardPanelData.data.queryType == "sql"
        ) {
          // Call the updateQueryValue function
          updateQueryValue();
        } else {
          // auto query mode selected
          // remove the custom fields from the list
          dashboardPanelData.meta.stream.customQueryFields = [];
        }
        // if (dashboardPanelData.data.queryType == "promql") {
        //     updatePromQLQuery()
        // }
      },
      { deep: true }
    );

    // on queryerror change dispatch resize event to resize monaco editor
    watch(
      () => dashboardPanelData.meta.errors.queryErrors,
      () => {
        window.dispatchEvent(new Event("resize"));
      }
    );

    // This function parses the custom query and generates the errors and custom fields
    const updateQueryValue = () => {
      // store the query in the dashboard panel data
      // dashboardPanelData.meta.editorValue = value;
      // dashboardPanelData.data.query = value;

      if (
        dashboardPanelData.data.queries[
          dashboardPanelData.layout.currentQueryIndex
        ].customQuery &&
        dashboardPanelData.data.queryType != "promql"
      ) {
        // empty the errors
        dashboardPanelData.meta.errors.queryErrors = [];

        // Get the parsed query
        try {
          dashboardPanelData.meta.parsedQuery = parser.astify(
            dashboardPanelData.data.queries[
              dashboardPanelData.layout.currentQueryIndex
            ].query
          );
        } catch (e) {
          // exit as there is an invalid query
          dashboardPanelData.meta.errors.queryErrors.push("Invalid SQL Syntax");
          return null;
        }
        if (!dashboardPanelData.meta.parsedQuery) {
          return;
        }

        // We have the parsed query, now get the columns and tables
        // get the columns first
        if (
          Array.isArray(dashboardPanelData.meta.parsedQuery?.columns) &&
          dashboardPanelData.meta.parsedQuery?.columns?.length > 0
        ) {
          const oldCustomQueryFields = JSON.parse(
            JSON.stringify(dashboardPanelData.meta.stream.customQueryFields)
          );
          dashboardPanelData.meta.stream.customQueryFields = [];
          dashboardPanelData.meta.parsedQuery.columns.forEach(
            (item: any, index: any) => {
              let val;
              // if there is a lable, use that, else leave it
              if (item["as"] === undefined || item["as"] === null) {
                val = item["expr"]["column"];
              } else {
                val = item["as"];
              }
              if (
                !dashboardPanelData.meta.stream.customQueryFields.find(
                  (it) => it.name == val
                )
              ) {
                dashboardPanelData.meta.stream.customQueryFields.push({
                  name: val,
                  type: "",
                });
              }
            }
          );

          // update the existing x and y axis fields
          updateXYFieldsOnCustomQueryChange(oldCustomQueryFields);
        } else {
          dashboardPanelData.meta.errors.queryErrors.push("Invalid Columns");
        }

        // now check if the correct stream is selected
        if (dashboardPanelData.meta.parsedQuery.from?.length > 0) {
          const streamFound = dashboardPanelData.meta.stream.streamResults.find(
            (it) => it.name == dashboardPanelData.meta.parsedQuery.from[0].table
          );
          if (streamFound) {
            if (
              dashboardPanelData.data.queries[
                dashboardPanelData.layout.currentQueryIndex
              ].fields.stream != streamFound.name
            ) {
              dashboardPanelData.data.queries[
                dashboardPanelData.layout.currentQueryIndex
              ].fields.stream = streamFound.name;
            }
          } else {
            dashboardPanelData.meta.errors.queryErrors.push("Invalid stream");
          }
        } else {
          dashboardPanelData.meta.errors.queryErrors.push(
            "Stream name required"
          );
        }
      }
    };

    const updatePromQLQuery = async (event, value) => {
      autoCompleteData.value.query = value;
      autoCompleteData.value.text = event.changes[0].text;
      autoCompleteData.value.dateTime = {
        startTime: dashboardPanelData.meta.dateTime.start_time?.getTime(),
        endTime: dashboardPanelData.meta.dateTime.end_time?.getTime(),
      };
      autoCompleteData.value.position.cursorIndex =
        queryEditorRef.value.getCursorIndex();
      autoCompleteData.value.popup.open =
        queryEditorRef.value.triggerAutoComplete;
      autoCompleteData.value.popup.close =
        queryEditorRef.value.disableSuggestionPopup;
      getSuggestions();
    };

    const onUpdateToggle = (value) => {
      dashboardPanelData.meta.errors.queryErrors = [];
    };

    return {
      t,
      router,
      updateQueryValue,
      updatePromQLQuery,
      onDropDownClick,
      promqlMode,
      dashboardPanelData,
      confirmQueryModeChangeDialog,
      onUpdateToggle,
      addTab,
      removeTab,
      currentQuery,
      autoCompleteData,
      autoCompletePromqlKeywords,
      getSuggestions,
      queryEditorRef,
      updateQuery,
    };
  },
});
</script>

<style lang="scss" scoped>
.sql-bar {
  height: 40px !important;
  // overflow: hidden;
  cursor: pointer;
}
.q-ml-sm:hover {
  background-color: #eaeaeaa5;
  border-radius: 50%;
}

// .query-tabs-container {
//   width: 100%;
//   display: flex;
//   flex-direction: row;
//   justify-content: flex-start;
//   align-items: center;
// }

// .query-tab {
//   display: flex;
//   flex-direction: row;
//   align-items: center;
//   margin-right: 10px;
//   padding: 5px;

//   &:hover {
//         background-color: #eaeaeaa5;
//     }
// }

// .tab-label {
//   margin-right: 5px;
// }

// .remove-button {
//   cursor: pointer;
//   display: flex;
//   align-items: center;
//   justify-content: center;
//   width: 20px;
//   height: 20px;
// }

// .query-tab.active {
//     border-bottom: 3px solid #000;
// }
</style>
