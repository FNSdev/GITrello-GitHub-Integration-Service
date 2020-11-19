def tag

pipeline {
    environment {
        IMAGE_NAME = 'fnsdev/gitrello-github-integration-service'
        BOT_TOKEN = credentials('bot-token')
        CHAT_ID = '-1001347488559'
    }
    agent {
        kubernetes {
            defaultContainer 'jnlp'
            yaml """
              apiVersion: v1
              kind: Pod
              metadata:
                name: ci
                labels:
                  app: jenkins
              spec:
                nodeSelector:
                  type: ci
                containers:
                  - name: rust
                    image: rust:1.47
                    command:
                      - cat
                    tty: true
                    resources:
                      requests:
                        memory: "400Mi"
                        cpu: "0.5"
                      limits:
                        memory: "400Mi"
                        cpu: "0.5"
                  - name: docker
                    image: docker:19.03
                    command:
                      - cat
                    tty: true
                    resources:
                      requests:
                        memory: "200Mi"
                        cpu: "0.2"
                      limits:
                        memory: "200Mi"
                        cpu: "0.2"
                    volumeMounts:
                      - name: dockersock
                        mountPath: /var/run/docker.sock
                  - name: helm
                    image: lachlanevenson/k8s-helm:v3.3.1
                    command:
                      - cat
                    tty: true
                volumes:
                  - name: dockersock
                    hostPath:
                      path: /var/run/docker.sock
            """
        }
    }
    stages {
        stage ('Set tag') {
            steps {
                script {
                    sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Build started $BUILD_URL %F0%9F%AA%92'"
                    tag = sh(script: 'git describe', returnStdout: true).trim()
                }
            }
        }
        stage('Build image') {
            steps {
                script {
                    sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Building Docker image %F0%9F%AA%92'"
                }
                container('docker') {
                    sh "docker build -t ${IMAGE_NAME}:${tag} ."
                }
            }
        }
        stage('Publish image') {
            steps {
                container('docker') {
                    withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'USERNAME', passwordVariable: 'PASSWORD')]) {
                        sh "echo ${PASSWORD} | docker login -u ${USERNAME} --password-stdin"
                        sh "docker push ${IMAGE_NAME}:${tag}"
                    }
                }
            }
        }
        stage('Migrate database') {
            environment {
                DATABASE_URL = credentials('github-integration-service-db-url')
            }
            steps {
                script {
                    sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Migrating database %F0%9F%AA%92'"
                }
                container('rust') {
                    sh """
                      apt-get update && \
                        apt-get install --no-install-recommends -y libpq-dev && \
                        apt-get clean && \
                        rm -rf /var/lib/apt/lists/*
                    """
                    sh "cargo install diesel_cli --no-default-features --features postgres"
                    sh "diesel migration run"
                }
            }
        }
        stage('Deploy chart') {
            steps {
                script {
                    sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Deploying helm chart %F0%9F%AA%92'"
                }
                container('helm') {
                    withCredentials([file(credentialsId: 'github-integration-service-overrides', variable: 'OVERRIDES')]) {
                        sh "helm upgrade gitrello-github-integration-service --install ./manifests/gitrello-github-integration-service -f ${OVERRIDES} --set deployment.image.tag=${tag}"
                    }
                }
            }
        }
    }
    post {
        success {
            script {
               sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Build succeeded %F0%9F%AA%92'"
            }
        }
        failure {
            script {
                sh "curl -s -X POST https://api.telegram.org/bot$BOT_TOKEN/sendMessage -d chat_id=$CHAT_ID -d text='%F0%9F%AA%92 Build failed %F0%9F%AA%92'"
            }
        }
    }
}
