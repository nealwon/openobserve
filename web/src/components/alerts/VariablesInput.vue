<template>
  <div class="col-12 q-py-sm">
    <template v-if="!variables.length">
      <q-btn
        label="Add Variable"
        size="sm"
        class="text-bold add-variable"
        icon="add"
        style="
          border-radius: 4px;
          text-transform: capitalize;
          background: #f2f2f2 !important;
          color: #000 !important;
        "
        @click="addVariable"
      />
    </template>
    <template v-else>
      <div
        v-for="(variable, index) in (variables as any)"
        :key="variable.uuid"
        class="q-col-gutter-sm q-pb-sm flex items-center"
      >
        <div class="q-ml-none">
          <q-input
            :data-test="`add-destination-header-${variable['key']}-key-input`"
            v-model="variable.key"
            color="input-border"
            bg-color="input-bg"
            class="showLabelOnTop"
            stack-label
            outlined
            filled
            :placeholder="t('common.name')"
            dense
            tabindex="0"
            style="min-width: 250px"
          />
        </div>
        <div class="q-ml-none">
          <q-input
            :data-test="`add-destination-header-${variable['key']}-value-input`"
            v-model="variable.value"
            :placeholder="t('common.value')"
            color="input-border"
            bg-color="input-bg"
            class="showLabelOnTop"
            stack-label
            outlined
            filled
            dense
            isUpdatingDestination
            tabindex="0"
            style="min-width: 250px"
          />
        </div>
        <div class="col-2 q-ml-none">
          <q-btn
            :data-test="`add-destination-header-${variable['key']}-delete-btn`"
            :icon="outlinedDelete"
            class="q-ml-xs iconHoverBtn"
            padding="sm"
            unelevated
            size="sm"
            round
            flat
            :title="t('alert_templates.edit')"
            @click="removeVariable(variable)"
          />
          <q-btn
            data-test="add-destination-add-header-btn"
            v-if="index === variables.length - 1"
            icon="add"
            class="q-ml-xs iconHoverBtn"
            padding="sm"
            unelevated
            size="sm"
            round
            flat
            :title="t('alert_templates.edit')"
            @click="addVariable"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { defineProps } from "vue";
import { useI18n } from "vue-i18n";
import { outlinedDelete } from "@quasar/extras/material-icons-outlined";

const props = defineProps({
  variables: {
    type: Array,
    required: true,
  },
});

const emits = defineEmits(["add:variable", "remove:variable"]);

const { t } = useI18n();

const removeVariable = (variable: any) => {
  emits("remove:variable", variable);
};

const addVariable = () => {
  emits("add:variable");
};
</script>

<style lang="scss">
.add-variable {
  .q-icon {
    margin-right: 4px !important;
    font-size: 15px !important;
  }
}
</style>
