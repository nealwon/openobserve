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
  <!-- 
    Create a table like SearchResults.vue component.
    1. Props for columns, rows and loading
    2. use virtual scroll
    3. Column should have boolean property to close column. closable: true
    4. Column should have boolean property to hide column. hidden: true
    5. Component should have highlight property to highlight search text
    6. Rows should have boolean property to expand row. expandable: true
   -->
  <div class="q-pa-md">
    <q-table
      style="height: calc(100vh - 240px)"
      flat
      bordered
      ref="tableRef"
      :title="title"
      :rows="rows"
      :columns="columns as []"
      :table-colspan="9"
      row-key="index"
      virtual-scroll
      :virtual-scroll-item-size="48"
      :rows-per-page-options="[0]"
      @virtual-scroll="onScroll"
      hide-bottom
    >
      <template v-slot:header="props">
        <q-tr :props="props" class="thead-sticky">
          <q-th
            v-for="col in props.cols"
            :key="col.name"
            :props="props"
            :style="col.style"
          >
            {{ col.label }}
          </q-th>
        </q-tr>
      </template>

      <template v-slot:body="props">
        <q-tr :props="props" :key="`m_${props.row.index}`">
          <q-td
            v-for="col in props.cols"
            :key="col.name"
            :props="props"
            @click="handleDataClick(col.name, props.row)"
          >
            <template v-if="col.slot">
              <slot :name="col.slotName" />
            </template>
            <template v-else-if="col.type === 'action'">
              <q-icon :name="col.icon" size="24px" class="cursor-pointer" />
            </template>
            <template v-else>
              {{ col.value }}
            </template>
          </q-td>
        </q-tr>
        <q-tr
          v-show="props.expand"
          :props="props"
          :key="`e_${props.row.index}`"
          class="q-virtual-scroll--with-prev"
        >
          <q-td colspan="100%">
            <div class="text-left">
              This is expand slot for row above: {{ props.row.name }} (Index:
              {{ props.row.index }}).
            </div>
          </q-td>
        </q-tr>
      </template>
    </q-table>
  </div>
</template>

<script setup lang="ts">
defineProps({
  columns: {
    type: Array,
    required: true,
  },
  rows: {
    type: Array,
    required: true,
  },
  loading: {
    type: Boolean,
    required: false,
    default: false,
  },
  highlight: {
    type: Boolean,
    required: false,
    default: false,
  },
  highlightText: {
    type: String,
    default: "",
  },
  title: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["event-emitted"]);

const handleDataClick = (columnName: string, row: any) => {
  emit("event-emitted", "cell-click", { columnName, row });
};

const onScroll = () => {};
</script>

<style>
.thead-sticky,
.tfoot-sticky {
  position: sticky;
  top: 0;
  opacity: 1;
  z-index: 1;
  background-color: #f5f5f5;
}
</style>
