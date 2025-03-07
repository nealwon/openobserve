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
  <div class="q-pl-xs q-pt-sm">
    <template v-for="span in spans as any[]" :key="span.spanId">
      <div :style="{ position: 'relative', width: '100%', overflow: 'hidden' }">
        <div
          :style="{
            height: spanDimensions.textHeight - 8 + 'px',
            margin: `4px 0px 4px ${
              span.hasChildSpans
                ? span.style.left
                : parseInt(span.style.left) +
                  spanDimensions.collapseWidth +
                  'px'
            }`,
          }"
          class="flex items-center no-wrap justify-start ellipsis"
          :title="span.operationName"
        >
          <div
            v-if="span.hasChildSpans"
            :style="{
              width: spanDimensions.collapseWidth + 'px',
              height: spanDimensions.collapseHeight + 'px',
            }"
            class="flex justify-center items-center collapse-container cursor-pointer"
            @click.stop="toggleSpanCollapse(span.spanId)"
          >
            <q-icon
              dense
              round
              flat
              name="expand_more"
              class="collapse-btn"
              :style="{
                rotate: collapseMapping[span.spanId] ? '0deg' : '270deg',
              }"
            />
          </div>
          <div
            class="ellipsis q-ml-xs cursor-pointer"
            :style="{
              paddingLeft: '4px',
              borderLeft: `3px solid ${span.style.color}`,
            }"
            @click="selectSpan(span.spanId)"
          >
            <span class="text-subtitle2 text-bold q-mr-sm">
              {{ span.serviceName }}
            </span>
            <span
              class="text-body2"
              :class="
                store.state.theme === 'dark'
                  ? 'text-grey-5'
                  : 'text-blue-grey-9'
              "
              >{{ span.operationName }}</span
            >
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { getImageURL } from "@/utils/zincutils";
import useTraces from "@/composables/useTraces";
import { useStore } from "vuex";

export default defineComponent({
  name: "TraceTree",
  props: {
    spans: {
      type: Array,
      default: () => [],
    },
    isCollapsed: {
      type: Boolean,
      default: false,
    },
    collapseMapping: {
      type: Object,
      default: () => {},
    },
    baseTracePosition: {
      type: Object,
      default: () => null,
    },
    depth: {
      type: Number,
      default: 0,
    },
    spanDimensions: {
      type: Object,
      default: () => {},
    },
  },
  emits: ["toggleCollapse"],
  setup(props, { emit }) {
    const { searchObj } = useTraces();

    const store = useStore();

    function toggleSpanCollapse(spanId: number | string) {
      emit("toggleCollapse", spanId);
    }

    const selectSpan = (spanId: string | null) => {
      searchObj.data.traceDetails.showSpanDetails = true;
      searchObj.data.traceDetails.selectedSpanId = spanId;
    };

    return {
      toggleSpanCollapse,
      getImageURL,
      selectSpan,
      store,
    };
  },
});
</script>

<style scoped lang="scss">
.spans-container {
  position: relative;
}

.collapse-btn {
  width: 10px;
  height: 10px;
}
</style>
