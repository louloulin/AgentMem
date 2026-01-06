{{/*
AgentMem Helm Chart 辅助模板
提供可重用的模板函数和标签定义
*/}}

{{/*
展开 chart 的完整名称
*/}}
{{- define "agentmem.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
创建 chart 名称和版本，用作 chart 标签
*/}}
{{- define "agentmem.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
通用标签
*/}}
{{- define "agentmem.labels" -}}
helm.sh/chart: {{ include "agentmem.chart" . }}
{{ include "agentmem.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
选择器标签
*/}}
{{- define "agentmem.selectorLabels" -}}
app.kubernetes.io/name: {{ include "agentmem.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
创建 chart 名称
*/}}
{{- define "agentmem.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
创建 ServiceAccount 名称
*/}}
{{- define "agentmem.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "agentmem.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
返回正确的镜像名称
*/}}
{{- define "agentmem.image" -}}
{{- $registryName := .Values.image.registry -}}
{{- $repositoryName := .Values.image.repository -}}
{{- $tag := .Values.image.tag | toString -}}
{{- if .Values.global }}
    {{- if .Values.global.imageRegistry }}
     {{- $registryName = .Values.global.imageRegistry -}}
    {{- end -}}
{{- end -}}
{{- if $registryName }}
{{- printf "%s/%s:%s" $registryName $repositoryName $tag -}}
{{- else -}}
{{- printf "%s:%s" $repositoryName $tag -}}
{{- end -}}
{{- end -}}

{{/*
返回 PostgreSQL 主机名
*/}}
{{- define "agentmem.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "agentmem.fullname" .) -}}
{{- else }}
{{- .Values.externalDatabase.host -}}
{{- end }}
{{- end }}

{{/*
返回 PostgreSQL 端口
*/}}
{{- define "agentmem.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.service.port | default 5432 -}}
{{- else }}
{{- .Values.externalDatabase.port | default 5432 -}}
{{- end }}
{{- end }}

{{/*
返回 PostgreSQL 数据库名
*/}}
{{- define "agentmem.postgresql.database" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database | default "agentmem" -}}
{{- else }}
{{- .Values.externalDatabase.database | default "agentmem" -}}
{{- end }}
{{- end }}

{{/*
返回 PostgreSQL 用户名
*/}}
{{- define "agentmem.postgresql.username" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.username | default "agentmem" -}}
{{- else }}
{{- .Values.externalDatabase.username | default "agentmem" -}}
{{- end }}
{{- end }}

{{/*
返回 PostgreSQL 密码 Secret 名称
*/}}
{{- define "agentmem.postgresql.secretName" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "agentmem.fullname" .) -}}
{{- else }}
{{- printf "%s-externaldb" (include "agentmem.fullname" .) -}}
{{- end }}
{{- end }}

{{/*
返回 Redis 主机名
*/}}
{{- define "agentmem.redis.host" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis-master" (include "agentmem.fullname" .) -}}
{{- else }}
{{- .Values.externalRedis.host -}}
{{- end }}
{{- end }}

{{/*
返回 Redis 端口
*/}}
{{- define "agentmem.redis.port" -}}
{{- if .Values.redis.enabled }}
{{- .Values.redis.master.service.port | default 6379 -}}
{{- else }}
{{- .Values.externalRedis.port | default 6379 -}}
{{- end }}
{{- end }}

{{/*
返回 Redis 密码 Secret 名称
*/}}
{{- define "agentmem.redis.secretName" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis" (include "agentmem.fullname" .) -}}
{{- else }}
{{- printf "%s-externalredis" (include "agentmem.fullname" .) -}}
{{- end }}
{{- end }}

{{/*
返回 Elasticsearch 主机名
*/}}
{{- define "agentmem.elasticsearch.host" -}}
{{- if .Values.elasticsearch.enabled }}
{{- printf "%s-elasticsearch" (include "agentmem.fullname" .) -}}
{{- else }}
{{- .Values.externalElasticsearch.host -}}
{{- end }}
{{- end }}

{{/*
返回 Elasticsearch 端口
*/}}
{{- define "agentmem.elasticsearch.port" -}}
{{- if .Values.elasticsearch.enabled }}
{{- .Values.elasticsearch.service.port | default 9200 -}}
{{- else }}
{{- .Values.externalElasticsearch.port | default 9200 -}}
{{- end }}
{{- end }}

{{/*
返回环境变量配置
*/}}
{{- define "agentmem.env" -}}
- name: DATABASE_URL
  value: "postgresql://{{ include "agentmem.postgresql.username" . }}:$(POSTGRES_PASSWORD)@{{ include "agentmem.postgresql.host" . }}:{{ include "agentmem.postgresql.port" . }}/{{ include "agentmem.postgresql.database" . }}"
- name: POSTGRES_PASSWORD
  valueFrom:
    secretKeyRef:
      name: {{ include "agentmem.postgresql.secretName" . }}
      key: password
- name: REDIS_URL
  value: "redis://:$(REDIS_PASSWORD)@{{ include "agentmem.redis.host" . }}:{{ include "agentmem.redis.port" . }}/0"
- name: REDIS_PASSWORD
  valueFrom:
    secretKeyRef:
      name: {{ include "agentmem.redis.secretName" . }}
      key: password
{{- if or .Values.elasticsearch.enabled .Values.externalElasticsearch.host }}
- name: ELASTICSEARCH_URL
  value: "http://{{ include "agentmem.elasticsearch.host" . }}:{{ include "agentmem.elasticsearch.port" . }}"
{{- end }}
- name: LOG_LEVEL
  value: {{ .Values.config.logLevel | quote }}
- name: SERVER_PORT
  value: {{ .Values.service.targetPort | quote }}
- name: RUST_LOG
  value: {{ .Values.config.rustLog | quote }}
{{- if .Values.config.extraEnv }}
{{- toYaml .Values.config.extraEnv }}
{{- end }}
{{- end }}

{{/*
返回 Pod 反亲和性配置
*/}}
{{- define "agentmem.podAntiAffinity" -}}
{{- if eq .Values.podAntiAffinity "hard" }}
podAntiAffinity:
  requiredDuringSchedulingIgnoredDuringExecution:
  - labelSelector:
      matchLabels: {{- include "agentmem.selectorLabels" . | nindent 8 }}
    topologyKey: kubernetes.io/hostname
{{- else if eq .Values.podAntiAffinity "soft" }}
podAntiAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
  - weight: 100
    podAffinityTerm:
      labelSelector:
        matchLabels: {{- include "agentmem.selectorLabels" . | nindent 10 }}
      topologyKey: kubernetes.io/hostname
{{- end }}
{{- end }}

