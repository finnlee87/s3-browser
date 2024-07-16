<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { ref, reactive } from "vue";
import { invoke } from '@tauri-apps/api/tauri'
import { Icon } from '@iconify/vue'
import { open } from '@tauri-apps/api/dialog'

const dialogFormVisible = ref(false);

const openLogin = () => {
  dialogFormVisible.value = true;
}

const formRef = ref(null);
const form = reactive({
  endpointUrl: '',
  accessKey: '',
  secretKey: '',
})


const showFlag = ref(true);
const treeData = ref([]);

const loadData = async () => {
  try {
    console.log('loadData');
    let data = [];
    let buckets = await invoke('list_buckets', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl });
    console.log(buckets);
    for (let bucket of buckets) {
      data.push({
        type: 'folder',
        path: `${bucket}/`,
        date: '-',
        objType: 'bucket',
        size: '-'
      });
      let objects = await invoke('list_objects', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, bucket: bucket });
      for (let objectData of objects) {
        let parts = objectData.split(',');
        data.push({
          type: parts[0].endsWith('/') ? 'folder' : 'file',
          path: `${bucket}/${parts[0]}`,
          date: parts[1],
          objType: 'object',
          size: parts[2] === '0B' ? '-' : parts[2],
        });
      }
    }

    treeData.value = buildTree(data);
    console.log(buildTree(data));
    showFlag.value = false;
    dialogFormVisible.value = false;
    return true;
  } catch(e) {
    console.log(e.toString().indexOf('HTTP 403'));
    console.error(e);
    if (e.toString().indexOf('HTTP 403') > -1) {
      ElMessage.error('Access key or secret key is incorrect.')
    }
    open();
  }
  return false;
}
const login = () => {
  formRef.value.validate(async (valid) => {
    if (valid) {
      localStorage.setItem('endpointUrl', form.endpointUrl);
      localStorage.setItem('accessKey', form.accessKey);
      localStorage.setItem('secretKey', form.secretKey);
      await loadData();
    }
  })
}

if (!localStorage.getItem('endpointUrl')) {
  openLogin();
} else {
  form.accessKey = localStorage.getItem('accessKey');
  form.secretKey = localStorage.getItem('secretKey');
  form.endpointUrl = localStorage.getItem('endpointUrl');
  loadData();
}


const menuVisible = ref(false);
const menuPosition = reactive({ x: 0, y: 0 });
const currentRow = ref(null);
const handleRowContextMenu = (row, _col, event) => {
  event.preventDefault();
  menuPosition.x = event.clientX;
  menuPosition.y = event.clientY;
  menuVisible.value = true;
  currentRow.value = {
    path: row.path,
    objType: row.objType,
    hasChildren: row.children && row.children.length > 0,
  };
  document.addEventListener('click', handleClickOutside);
}
function handleClickOutside() {
  menuVisible.value = false;
  document.removeEventListener('click', handleClickOutside);
}

async function handleMenuAction(command) {
  menuVisible.value = false;
  switch (command) {
    case 'Create folder':
      const folderName = await ElMessageBox.prompt('Please input folder name', '', {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        inputPattern: /^[0-9a-zA-Z]+$/,
        inputErrorMessage: 'Invalid name',
      });

      try {
        if (folderName) {
          await invoke('create_folder', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, path: currentRow.value.path, folderName: folderName.value});
          await loadData();
        }
      } catch(e) {
        console.error(e);
        ElMessage.error(error);
      }

      // const selected = await open({
      //   directory: true,
      // });
      // console.log(selected);
      // if (Array.isArray(selected)) {
      //   // user selected multiple directories
      // } else if (selected === null) {
      //   // user cancelled the selection
      // } else {
      //   // user selected a single directory
      // }
      break;
    case 'upload':
      let selectedFile = await open({
        directory: false,
      });
      if (selectedFile) {
        const loading = ElLoading.service({
          lock: true,
          text: 'Loading',
          background: 'rgba(0, 0, 0, 0.7)',
        })
        try {   
          await invoke('upload_file', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, folderPath: currentRow.value.path, filePath: selectedFile });
          await loadData();
          loading.close();
        } catch(e) {
          console.error(e);
          loading.close()
          ElMessage.error(error);
        }
      }
      break;
    case 'download':
      let selectedFolder = await open({
        directory: true,
      });
      if (selectedFolder) {
        const loading = ElLoading.service({
          lock: true,
          text: 'Loading',
          background: 'rgba(0, 0, 0, 0.7)',
        })
        try {   
          await invoke('download_file', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, filePath: currentRow.value.path, targetPath: selectedFolder });
          loading.close();
        } catch(e) {
          console.error(e);
          loading.close();
          ElMessage.error(error);
        }
      }
      break;
    case 'delete':
      if (currentRow.value.objType === 'bucket') {
        try {
          await invoke('delete_bucket', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, bucketName: currentRow.value.path.substring(0, currentRow.value.path.length - 1) });
          await loadData();
        } catch(e) {
          console.error(e);
          ElMessage.error(error);
        }
      } else {
        try {
          await invoke('delete_object', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, objPath: currentRow.value.path });
          await loadData();
        } catch(e) {
          console.error(e);
          ElMessage.error(error);
        }
      }
      break;
  }
}

function buildTree(paths) {
  const root = {};

  paths.forEach(({ path, type, date, objType, size }) => {
    const parts = path.split('/').filter(Boolean);
    let current = root;

    parts.forEach((part, index) => {
      if (!current[part]) {
        if (index === parts.length - 1) {
          current[part] = { path, type, date, objType, size: type === 'folder' ? '-' : size, children: type === 'folder' ? [] : null };
        } else {
          current[part] = current[part] || { path: parts.slice(0, index + 1).join('/') + '/', type: 'folder', date, objType, size: '-', children: {} };
        }
      }
      current = current[part].children || {};
    });
  });

  function convertToTree(node) {
    if (!node || !Object.keys(node).length) return [];
    const result = [];
    for (const [key, value] of Object.entries(node)) {
      result.push({
        name: key,
        path: value.path,
        type: value.type,
        date: value.date,
        objType: value.objType,
        size: value.size,
        children: value.type === 'folder' ? convertToTree(value.children) : null
      });
    }
    return result;
  }

  return convertToTree(root);
}

const rules = reactive({
  endpointUrl: [
    { required: true, message: 'Please input endpoint', trigger: 'blur' },
  ],
  accessKey: [
    { required: true, message: 'Please input access key', trigger: 'blur' },
  ],
  secretKey: [
    { required: true, message: 'Please input secret key', trigger: 'blur' },
  ],
})

const switchAccount = async () => {
  showFlag.value = true;
}

const createBucket = async () => {
  bucketDialogVisible.value = true;
}

const bucketDialogVisible = ref(false);
const bucketForm = reactive({
  name: '',
  region: '',
});

const bucketRules = reactive({
  name: [
    { required: true, message: 'Please input bucket name', trigger: 'blur' },
  ],
})

const bucketFormRef = ref(null);

const createBucketAction = async () => {
  bucketFormRef.value.validate(async (valid) => {
    if (valid) {
      try {
        await invoke('create_bucket', { accessKey: form.accessKey, secretKey: form.secretKey, endpoint: form.endpointUrl, bucketName: bucketForm.name, region: bucketForm.region });
        await loadData();
      } catch(error) {
        console.error(error);
        ElMessage.error(error);
      }
    }
  })
}

</script>

<template>
  <div class="container">
    <div v-if="showFlag" style="padding-top: 10vh;">
      <h1>Welcome to S3 Browser!</h1>
      <el-row>
        <el-col :span="24" style="text-align: center;">
          <el-button size="large" type="primary" @click="openLogin">Login to S3</el-button>
        </el-col>
      </el-row>
    </div>
    <div v-else>
      <div style="padding: 10px 0; text-align: right;">
        <el-button type="primary" @click="switchAccount">Logout</el-button>
        <el-button type="primary" @click="createBucket">Create Bucket</el-button>
      </div>
      <el-table :data="treeData" style="width: 100%" row-key="path" @row-contextmenu="handleRowContextMenu">
        <el-table-column label="Name">
          <template #default="scope">
            <div style="display: inline-flex; align-items: center;">
              <Icon icon="bi:database" v-if="scope.row.objType === 'bucket'"/>
              <Icon icon="bi:files" v-if="scope.row.objType === 'object' && scope.row.type === 'file'"/>
              <Icon icon="bi:folder" v-if="scope.row.objType === 'object' && scope.row.type === 'folder'"/>
              <div style="display:inline-block; margin-left: 2px; font-size: 14px; padding: 2px;">{{ scope.row.name }}</div>
            </div>
            
          </template>
        </el-table-column>
        <el-table-column prop="size" label="Size" width="100"></el-table-column>
        <el-table-column prop="date" label="Date" width="220"></el-table-column>
      </el-table>
      <div v-if="menuVisible" :style="{ top: `${menuPosition.y}px`, left: `${menuPosition.x}px` }" class="context-menu">
        <div class="context-menu-item" v-if="currentRow.path.endsWith('/')" @click="handleMenuAction('Create folder')">Create folder</div>
        <div class="context-menu-item" v-if="currentRow.path.endsWith('/')" @click="handleMenuAction('upload')">Upload file</div>
        <div class="context-menu-item" v-if="!currentRow.path.endsWith('/')" @click="handleMenuAction('download')">Download</div>
        <div class="context-menu-item" v-if="!currentRow.hasChildren" @click="handleMenuAction('delete')">Delete</div>
      </div>
    </div>
  </div>

  <el-dialog v-model="dialogFormVisible" title="Login Info" width="500">
    <el-form :model="form" :rules="rules" ref="formRef">
      <el-form-item label="Endpoint" :label-width="100" prop="endpointUrl">
        <el-input v-model="form.endpointUrl" autocomplete="off" />
      </el-form-item>
      <el-form-item label="AccessKey" :label-width="100" prop="accessKey">
        <el-input v-model="form.accessKey" autocomplete="off" />
      </el-form-item>
      <el-form-item label="SecretKey" :label-width="100" prop="secretKey">
        <el-input v-model="form.secretKey" autocomplete="off" type="password" />
      </el-form-item>
    </el-form>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="dialogFormVisible = false">Cancel</el-button>
        <el-button type="primary" @click="login">
          Confirm
        </el-button>
      </div>
    </template>
  </el-dialog>


  <el-dialog v-model="bucketDialogVisible" title="Create bucket" width="500">
    <el-form :model="bucketForm" :rules="bucketRules" ref="bucketFormRef">
      <el-form-item label="Name" :label-width="80" prop="name">
        <el-input v-model="bucketForm.name" autocomplete="off" />
      </el-form-item>
      <el-form-item label="Region" :label-width="80">
        <el-input v-model="bucketForm.region" autocomplete="off" />
      </el-form-item>
    </el-form>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="bucketDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="createBucketAction">
          Confirm
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.context-menu {
  position: fixed;
  background-color: white;
  border: 1px solid #ccc;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  z-index: 1000;
}
.context-menu-item {
  padding: 4px 0px 4px 8px;
  width: 120px;
  cursor: pointer;
  font-size: 14px;
  text-align: left;
}
.context-menu-item:hover {
  background-color: #f5f5f5;
}
</style>
