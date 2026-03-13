<script setup>
import DefaultTheme from 'vitepress/theme'
const { Layout } = DefaultTheme
</script>

<template>
  <Layout>
    <template #home-hero-after>
      <div class="custom-hero-image">
        <img src="/examples/cship.png" alt="CShip Status Line Preview" />
      </div>
    </template>
  </Layout>
</template>

<style scoped>
.custom-hero-image {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  margin-top: -48px; 
  padding: 0 24px 20px 24px; 
  box-sizing: border-box;
}

.custom-hero-image img {
  max-width: 1000px;
  width: 100%;
  height: auto;
  border-radius: 12px;
}

@media (max-width: 960px) {
  .custom-hero-image {
    margin-top: -32px;
  }
}

@media (max-width: 640px) {
  .custom-hero-image {
    margin-top: -32px;
  }
}
</style>