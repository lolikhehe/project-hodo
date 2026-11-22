import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";import"./ia-status-indicator-Bd61y7bd.js";import{t as f}from"./log-Bvw5tZC3.js";import{t as p}from"./model-C9UpV6Io.js";import"./ia-button-DYuVUpnO.js";import{t as m}from"./add-light-Bon_gAs-.js";function h(){return{SUCCESS:o(`Upload successful!`),INVALID_TYPE:o(`Image file must be a JPEG, PNG, or GIF.`),SIZE_EXCEEDED:e=>o(r`Image file must be less than ${e}MB.`),UPLOAD_FAILED:e=>o(r`Upload failed with status ${e}`),VERIFY_FAILED:e=>o(r`Verify failed with status ${e}`),WAITING_FOR_TASKS:e=>o(r`waiting for your ${e} tasks to finish`),WAITING_FOR_QUEUE:o(`waiting for your tasks to queue`),ADMIN_ERROR:o(`status task failure -- an admin will need to resolve`),RELOADING:o(`reloading page with your image`),ERROR_VERIFYING_UPLOAD:o(`Error verifying upload`),UPLOAD_FAILED_GENERIC:o(`Upload failed`)}}var g=class extends c{constructor(...e){super(...e),this.src=``,this.variant=`full`,this.loading=!1,this.disabled=!1,this.hovering=!1,this.onDocumentDragOver=e=>{this.prevent(e),this.hovering=!0,this.dispatchEvent(new Event(`onDragOver`))},this.onDocumentDrop=e=>{this.prevent(e)}}firstUpdated(){document.addEventListener(`dragover`,this.onDocumentDragOver,!1),document.addEventListener(`drop`,this.onDocumentDrop,!1)}disconnectedCallback(){super.disconnectedCallback(),document.removeEventListener(`dragover`,this.onDocumentDragOver,!1),document.removeEventListener(`drop`,this.onDocumentDrop,!1)}openFileDialog(){this.disabled||this.fileInput.click()}emitFileSelectedEvent(e){this.dispatchEvent(new CustomEvent(`fileSelected`,{detail:e,bubbles:!0,composed:!0}))}prevent(e){e.preventDefault(),e.stopPropagation()}onClick(){this.openFileDialog()}onInputChange(){let e=this.fileInput.files;e&&e[0]&&(this.emitFileSelectedEvent(e[0]),this.fileInput.value=``)}onDrop(e){if(this.prevent(e),this.disabled)return;this.hovering=!1;let t=e.dataTransfer?.files;t&&t[0]&&this.emitFileSelectedEvent(t[0])}render(){return a`
      <div
        class="drop-zone ${this.variant}"
        role="button"
        tabindex="0"
        aria-disabled=${this.disabled?`true`:`false`}
        @click=${this.onClick}
        @keydown=${e=>(e.key===`Enter`||e.key===` `)&&this.onClick()}
        @dragover=${this.prevent}
        @drop=${this.onDrop}
        @mouseenter=${()=>{this.disabled||(this.hovering=!0,this.requestUpdate())}}
        @mouseleave=${()=>{this.hovering=!1,this.requestUpdate()}}
      >
        <img
          src=${this.src}
          alt="profile picture"
          class="profile-pic ${this.shouldShowCompactOverlay?`hovering`:``}"
        />

        ${this.fileInputTemplate}
        ${this.shouldShowCompactOverlay?this.compactOverlayTemplate:n}
      </div>
    `}get fileInputTemplate(){return a`<input
      type="file"
      id="file-input"
      accept="image/*"
      @change=${this.onInputChange}
    />`}get shouldShowCompactOverlay(){return this.variant===`compact`&&(this.hovering||this.loading)}get compactOverlayTemplate(){return a`<div class="overlay">
      ${this.loading?this.loadingTemplate:this.addIconTemplate}
    </div>`}get loadingTemplate(){return a`<ia-status-indicator mode="loading"></ia-status-indicator>`}get addIconTemplate(){return a`<img
      class="add-icon"
      src=${m}
      alt=${o(`Upload image`)}
    />`}static get styles(){return e`
      :host {
        display: inline-block;
        position: relative;
      }
      .drop-zone {
        position: relative;
        cursor: pointer;
        outline: none;
      }
      .profile-pic {
        max-width: 100px;
        max-height: 100px;
        object-fit: cover;
        transition:
          opacity 0.2s ease,
          box-shadow 0.2s ease;
      }
      .profile-pic.hovering {
        opacity: 0.6;
        box-shadow: 0 0 6px #666;
      }
      .compact .profile-pic {
        border-radius: var(--ia-drop-radius, 50%);
        height: 120px;
        width: 120px;
      }
      .overlay {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 2rem;
      }
      .overlay .add-icon {
        background: white;
        border-radius: 100%;
        border: 1px solid #ddd;
      }
      input[type='file'] {
        display: none;
      }
    `}};d([t({type:String})],g.prototype,`src`,void 0),d([t({type:String})],g.prototype,`variant`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`loading`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`disabled`,void 0),d([u(`#file-input`)],g.prototype,`fileInput`,void 0),d([s()],g.prototype,`hovering`,void 0),g=d([l(`pic-uploader-drop-zone`),i()],g);var _=class extends c{constructor(...e){super(...e),this.variant=`compact`,this.previewSrc=``,this.canSubmit=!1,this.disabled=!1,this.loading=!1}emitClosePreviewEvent(){this.dispatchEvent(new Event(`closePreview`))}emitSubmitClickEvent(){this.dispatchEvent(new Event(`submitClick`))}emitPickClickEvent(){this.dispatchEvent(new Event(`pickClick`))}render(){return a`
      <div
        class="centered ${this.loading?`loading`:``}"
        @dragover=${this.onDragOver}
        @drop=${this.onDrop}
      >
        ${this.variant===`compact`?this.compactTemplate:this.fullTemplate}
      </div>
    `}onDragOver(e){e.preventDefault(),e.stopPropagation()}onDrop(e){if(e.preventDefault(),e.stopPropagation(),this.disabled)return;let t=e.dataTransfer?.files;t&&t[0]&&this.dispatchEvent(new CustomEvent(`fileSelected`,{detail:t[0],bubbles:!0,composed:!0}))}get fullTemplate(){return a`
      <div class="full-variant">
        ${this.previewSrc?this.closeButtonTemplate:n}
        ${this.addIconTemplate}
        <div class="label">${o(`Drag & Drop an image file here or`)}</div>

        <div class="actions">
          <ia-button
            mode="primary"
            title="Remove selected image"
            @click=${this.emitPickClickEvent}
          >
            ${o(`Pick image to upload`)}
          </ia-button>

          ${this.previewSrc?this.postSelectionTemplate:n}
        </div>
      </div>
    `}get compactTemplate(){return a`
      <div class="compact-variant">
        <div class="actions">
          <input
            class="file-selector"
            name="file"
            type="file"
            accept="image/*"
          />
          <div class="select-message">
            <span>Drop a new image onto</span>
            <span>your picture here or</span>
            <ia-button
              mode="link"
              class=${this.loading?`pointer-none`:``}
              ?loading=${this.loading}
              @click=${this.emitPickClickEvent}
            >
              ${o(`select an image to upload`)}
            </ia-button>
          </div>
        </div>
      </div>
    `}get postSelectionTemplate(){return a`
      <ia-button
        class="custom"
        mode="custom"
        ?disabled=${this.disabled||!this.canSubmit}
        ?loading=${this.loading}
        @click=${this.emitSubmitClickEvent}
      >
        ${o(`Upload`)}
      </ia-button>

      ${this.loading?null:this.previewTemplate}
    `}get previewTemplate(){return a`<img
      class="preview-img"
      src=${this.previewSrc}
      alt=${o(`Selected image preview`)}
      @click=${this.emitClosePreviewEvent}
    />`}get closeButtonTemplate(){return a`<button
      class="close-btn"
      title="Remove selected image"
      @click=${()=>{this.emitClosePreviewEvent()}}
    >
      &#10005;
    </button>`}get addIconTemplate(){return a`<img
      class="add-icon"
      src=${m}
      alt=${o(`Upload image`)}
    />`}static get styles(){return e`
      .centered {
        padding: 1rem;
      }
      .full-variant {
        display: flex;
        flex-direction: column;
        text-align: inherit;
        gap: 1rem;
      }
      .compact-variant .file-selector {
        display: none;
      }
      .centered.loading {
        opacity: 0.8;
        pointer-events: none;
      }
      .add-icon {
        width: 2.5rem;
        height: 2.5rem;
        margin: 0 auto;
      }
      .select-message {
        display: flex;
        flex-direction: column;
        align-items: left;
      }
      .select-message a {
        cursor: pointer;
        color: var(--ia-link-color, #0066cc);
        text-decoration: none;
      }
      .preview-img {
        width: 12rem;
        height: 12rem;
        object-fit: cover;
        border-radius: 8px;
        box-shadow: 0 0 2px #aaa;
        margin-bottom: 0.5rem;
      }
      .preview-img:hover {
        cursor: pointer;
      }
      .actions {
        display: flex;
        gap: 1rem;
        align-items: center;
        justify-content: center;
        flex-direction: column;
      }
      .actions ia-button {
        --ia-theme-button-padding: 1rem;
      }
      .actions ia-button.custom {
        --ia-button-custom-border-color: green;
        --ia-button-custom-fill: green;
      }
      .label {
        font-weight: bold;
        font-size: 1.2rem;
        margin-bottom: 0.2rem;
      }
      .close-btn {
        position: absolute;
        top: 1rem;
        right: 1rem;
        background: transparent;
        border: none;
        font-size: 1.4rem;
        color: #888;
        cursor: pointer;
        padding: 0;
      }
      .close-btn:hover {
        color: #000;
      }
      @keyframes spin {
        to {
          transform: rotate(360deg);
        }
      }
    `}};d([t({type:String})],_.prototype,`variant`,void 0),d([t({type:String})],_.prototype,`previewSrc`,void 0),d([t({type:Boolean})],_.prototype,`canSubmit`,void 0),d([t({type:Boolean})],_.prototype,`disabled`,void 0),d([t({type:Boolean})],_.prototype,`loading`,void 0),_=d([l(`pic-uploader-controls`),i()],_);var v=class extends c{constructor(...e){super(...e),this.taskStatus=``,this.error=``,this.loading=!1}render(){return a` ${this.loading?this.statusTemplate:this.errorTemplate} `}get statusTemplate(){return a`<div class="status">${this.taskStatus}</div>`}get errorTemplate(){return a`<div class="error">${this.error}</div>`}static get styles(){return e`
      :host {
        display: grid;
        justify-items: center;
        gap: 1rem;
        padding: 0 1rem 1rem 1rem;
        width: 160px;
      }
      .error {
        color: #bb0505;
        font-size: 1.2rem;
        text-align: center;
        padding: 0 1rem;
      }
      .status {
        font-weight: 600;
        text-align: center;
        font-size: 1.2rem;
        user-select: none;
      }
    `}};d([t({type:String})],v.prototype,`taskStatus`,void 0),d([t({type:String})],v.prototype,`error`,void 0),d([t({type:Boolean})],v.prototype,`loading`,void 0),v=d([l(`pic-uploader-status`)],v);var y=class extends c{constructor(...e){super(...e),this.identifier=``,this.baseUrl=`archive.org`,this.picture=``,this.variant=`full`,this.lookingAtMyAccount=!1,this.maxFileSizeInMB=8,this.validFileTypes=[`image/jpeg`,`image/png`,`image/gif`],this.loading=!1,this.error=``,this.statusText=``,this.previewSrc=``,this.hasPendingPreview=!1,this.controlsVisible=!1,this.selectedFile=null}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServicesDependents(),this.variant===`compact`&&(this.controlsVisible=!0)}async setupAppServicesDependents(){let e=await this.appServices?.get();this.uploaderService=await e?.picUploaderService.get()}emitFileChangedEvent(){this.dispatchEvent(new CustomEvent(`fileChanged`,{detail:{error:this.error??``}}))}emitFileUploadedEvent(e){this.dispatchEvent(new CustomEvent(`fileUploaded`,{detail:{hasError:!!e.length}}))}async onFileSelected(e){let t=e.detail;this.error=``,f(`file selected`,t);let n=this.uploaderService?.validate({file:t,maxSizeMB:this.maxFileSizeInMB,validTypes:this.validFileTypes});if(n?.success){let e=await this.uploaderService?.readFileAsDataURL(t);e&&(this.previewSrc=e),this.selectedFile=t,this.controlsVisible=!0,this.hasPendingPreview=!0}else this.error=this.statusCodeToMessage(n?.code),this.statusText=``,this.previewSrc=``,this.selectedFile=null,this.hasPendingPreview=!1,this.controlsVisible=!0;this.emitFileChangedEvent()}async onSubmit(){if(this.selectedFile){this.loading=!0,this.error=``,this.statusText=h().WAITING_FOR_QUEUE,this.controlsVisible=!0;try{let e=await this.uploaderService?.upload({identifier:this.identifier,file:this.selectedFile});f(`upload response`,e),e?.success?(this.statusText=this.statusCodeToMessage(`waiting-for-queue`),this.variant===`full`&&await this.uploadVerify()):this.error=h().UPLOAD_FAILED_GENERIC}catch(e){f(e),this.error=h().UPLOAD_FAILED_GENERIC,this.statusText=``,this.controlsVisible=!0}finally{this.emitFileUploadedEvent(this.error),this.loading=!1,this.hasPendingPreview=!1,this.selectedFile=null}}}async uploadVerify(){let e=Math.round(Date.now()/1e3);return new Promise((t,n)=>{let r=setInterval(async()=>{try{let n=await this.uploaderService?.fetchUploadStatus({identifier:this.identifier,now:e});f(`verify poll response`,n),n?.code===p.VERIFY_FAILED?(this.error=h().ERROR_VERIFYING_UPLOAD,clearInterval(r),t()):n?.code===p.SUCCESS?(clearInterval(r),this.dropZone.src=this.previewSrc,this.controlsVisible=!1,t()):this.statusText=this.statusCodeToMessage(n?.code,n?.waitCount)}catch(e){clearInterval(r),this.error=h().ERROR_VERIFYING_UPLOAD,this.loading=!1,n(e)}},2e3)})}statusCodeToMessage(e,t=0){switch(e){case p.WAITING_FOR_QUEUE:return h().WAITING_FOR_QUEUE;case p.WAITING_FOR_TASKS:return h().WAITING_FOR_TASKS(t);case p.ADMIN_ERROR:return h().ADMIN_ERROR;case p.RELOADING:return h().RELOADING;case p.TYPE_ERROR:return h().INVALID_TYPE;case p.SIZE_ERROR:return h().SIZE_EXCEEDED(this.maxFileSizeInMB);default:return h().ERROR_VERIFYING_UPLOAD}}onPickClick(){this.dropZone?.openFileDialog?.()}onClosePreview(){this.previewSrc=``,this.error=``,this.selectedFile=null,this.hasPendingPreview=!1,this.controlsVisible=!1}onMouseEnter(){!this.previewSrc&&this.lookingAtMyAccount&&(this.controlsVisible=!0)}onMouseLeave(e,t){!e.relatedTarget?.closest(t)&&!this.previewSrc&&(this.controlsVisible=!1)}render(){let e=this.variant===`full`,t=e?this.picture:this.previewSrc||this.picture,r=e&&!this.lookingAtMyAccount||this.loading;return a`
      <div class="root ${this.variant}-panel">
        <pic-uploader-drop-zone
          .src=${t}
          .variant=${this.variant}
          ?loading=${this.loading}
          ?disabled=${r}
          @fileSelected=${this.onFileSelected}
          @onDragOver=${this.onMouseEnter}
          @mouseenter=${this.onMouseEnter}
          @mouseleave=${e=>this.onMouseLeave(e,`pic-uploader-drop-zone`)}
        ></pic-uploader-drop-zone>

        ${this.controlsVisible?this.uploadControlsTemplate:n}
      </div>
    `}get uploadControlsTemplate(){return a`
      <div class="controls ${this.loading?`busy`:``}">
        <pic-uploader-controls
          .variant=${this.variant}
          .previewSrc=${this.previewSrc}
          ?canSubmit=${this.hasPendingPreview&&!this.error}
          ?disabled=${this.loading}
          ?loading=${this.loading}
          @pickClick=${this.onPickClick}
          @submitClick=${this.onSubmit}
          @closePreview=${this.onClosePreview}
          @fileSelected=${this.onFileSelected}
          @mouseenter=${this.onMouseEnter}
          @mouseleave=${e=>this.onMouseLeave(e,`pic-uploader-controls`)}
        ></pic-uploader-controls>

        ${this.showUploadStatusTemplate?this.uploadStatusTemplate:n}
      </div>
    `}get showUploadStatusTemplate(){return this.variant===`full`&&(this.loading||this.error!==``)}get uploadStatusTemplate(){return a`
      <pic-uploader-status
        .loading=${this.loading}
        .taskStatus=${this.statusText}
        .error=${this.error}
      ></pic-uploader-status>
    `}static get styles(){return e`
      :host {
        --imgMaxHeight--: var(--imgMaxHeight, 100px);
        --imgMaxWidth--: var(--imgMaxWidth, 200px);
        --primaryTextColor--: var(--primaryTextColor, #2c2c2c);

        display: inline-grid;
        gap: 0.5rem;
        align-items: start;
        position: relative;
        color: var(--primaryTextColor--);
      }
      .compact-panel {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.5rem;
      }
      .full-panel .controls {
        position: absolute;
        top: 0;
        left: 0;
        z-index: 3;
        text-align: center;
        background: rgb(255, 255, 255);
        border: 3px solid rgb(204, 204, 204);
      }
      .controls.busy {
        opacity: 1;
      }
      .root.full file-drop-zone img {
        max-height: var(--imgMaxHeight--);
        max-width: var(--imgMaxWidth--);
        border-radius: 6px;
      }
    `}};d([t({type:Object})],y.prototype,`appServices`,void 0),d([t({type:String})],y.prototype,`identifier`,void 0),d([t({type:String})],y.prototype,`baseUrl`,void 0),d([t({type:String})],y.prototype,`picture`,void 0),d([t({type:String})],y.prototype,`variant`,void 0),d([t({type:Boolean})],y.prototype,`lookingAtMyAccount`,void 0),d([t({type:Number})],y.prototype,`maxFileSizeInMB`,void 0),d([t({type:Array})],y.prototype,`validFileTypes`,void 0),d([s()],y.prototype,`loading`,void 0),d([s()],y.prototype,`error`,void 0),d([s()],y.prototype,`statusText`,void 0),d([s()],y.prototype,`previewSrc`,void 0),d([s()],y.prototype,`hasPendingPreview`,void 0),d([s()],y.prototype,`controlsVisible`,void 0),d([u(`pic-uploader-drop-zone`)],y.prototype,`dropZone`,void 0),y=d([l(`ia-pic-uploader`)],y);